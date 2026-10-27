use crate::domain::auth::{ChangePasswordInput, CreateUserInput, User};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::UserRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::user_repository::SqliteUserRepository;
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use uuid::Uuid;

pub const MIN_PASSWORD_LEN: usize = 8;
pub const MAX_PASSWORD_LEN: usize = 128;

pub struct AuthService;

impl AuthService {
    pub fn has_users(db: &Db) -> AppResult<bool> {
        db.with_conn(|conn| Ok(SqliteUserRepository::new(conn).count()? > 0))
    }

    pub fn list_users(db: &Db) -> AppResult<Vec<User>> {
        db.with_conn(|conn| SqliteUserRepository::new(conn).list())
    }

    /// Creates a user. The first account may be created without authentication
    /// (that is the bootstrap step); afterwards `authenticated` must be true.
    pub fn create_user(db: &Db, input: &CreateUserInput, authenticated: bool) -> AppResult<User> {
        let username = validate_username(&input.username)?;
        let display_name = validate_display_name(&input.display_name)?;
        validate_password(&input.password)?;

        let password = input.password.clone();
        db.transaction(|conn| {
            let repo = SqliteUserRepository::new(conn);
            let is_first_user = repo.count()? == 0;

            if !is_first_user && !authenticated {
                return Err(AppError::unauthorized());
            }

            if repo.find_by_username(&username)?.is_some() {
                return Err(AppError::conflict(format!(
                    "The username \"{}\" is already taken",
                    username
                )));
            }

            let user = User {
                id: Uuid::new_v4().to_string(),
                username: username.clone(),
                display_name: display_name.clone(),
                is_active: true,
                created_at: crate::application::now_iso(),
                last_login_at: None,
            };
            repo.create(&user, &hash_password(&password)?)?;
            Ok(user)
        })
    }

    pub fn login(db: &Db, username: &str, password: &str) -> AppResult<User> {
        let username = username.trim();
        if username.is_empty() || password.is_empty() {
            return Err(AppError::validation("Enter your username and password"));
        }

        db.with_conn(|conn| {
            let repo = SqliteUserRepository::new(conn);
            let user = repo.find_by_username(username)?;

            let Some(user) = user else {
                // Spend the same amount of time as a real verification so an
                // attacker cannot tell existing usernames from missing ones.
                let _ = verify_password(
                    "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$LP0v0n3m0lQ0dQ5Yj1n0Q0z0n3m0lQ0dQ5Yj1n0Q0",
                    password,
                );
                return Err(AppError::new(
                    "INVALID_CREDENTIALS",
                    "Incorrect username or password",
                ));
            };

            if !user.is_active {
                return Err(AppError::forbidden("This account has been deactivated"));
            }

            let hash = repo
                .password_hash(&user.id)?
                .ok_or_else(|| AppError::new("INVALID_CREDENTIALS", "Incorrect username or password"))?;

            if !verify_password(&hash, password) {
                return Err(AppError::new(
                    "INVALID_CREDENTIALS",
                    "Incorrect username or password",
                ));
            }

            repo.record_login(&user.id, &crate::application::now_iso())?;
            Ok(user)
        })
    }

    pub fn change_password(db: &Db, user_id: &str, input: &ChangePasswordInput) -> AppResult<()> {
        validate_password(&input.new_password)?;
        let new_password = input.new_password.clone();

        db.with_conn(|conn| {
            let repo = SqliteUserRepository::new(conn);
            let hash = repo
                .password_hash(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))?;

            if !verify_password(&hash, &input.current_password) {
                return Err(AppError::new(
                    "INVALID_CREDENTIALS",
                    "Your current password is incorrect",
                ));
            }

            repo.update_password(user_id, &hash_password(&new_password)?)
        })
    }

    pub fn set_active(db: &Db, user_id: &str, is_active: bool) -> AppResult<()> {
        db.with_conn(|conn| SqliteUserRepository::new(conn).set_active(user_id, is_active))
    }

    /// Changes a user's username and display name, returning the updated user.
    pub fn update_user(
        db: &Db,
        user_id: &str,
        username: &str,
        display_name: &str,
    ) -> AppResult<User> {
        let username = validate_username(username)?;
        let display_name = validate_display_name(display_name)?;

        db.transaction(|conn| {
            let repo = SqliteUserRepository::new(conn);
            if repo.find_by_id(user_id)?.is_none() {
                return Err(AppError::not_found("User", user_id));
            }
            if let Some(existing) = repo.find_by_username(&username)? {
                if existing.id != user_id {
                    return Err(AppError::conflict(format!(
                        "The username \"{}\" is already taken",
                        username
                    )));
                }
            }
            repo.update_profile(user_id, &username, &display_name)
        })?;

        db.with_conn(|conn| {
            SqliteUserRepository::new(conn)
                .find_by_id(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))
        })
    }

    /// Removes a user. Payments they recorded keep their amounts but lose the
    /// attribution so the financial history stays intact.
    pub fn delete_user(db: &Db, user_id: &str) -> AppResult<()> {
        db.transaction(|conn| {
            let repo = SqliteUserRepository::new(conn);
            if repo.find_by_id(user_id)?.is_none() {
                return Err(AppError::not_found("User", user_id));
            }
            conn.execute(
                "UPDATE payments SET created_by = NULL WHERE created_by = ?",
                [user_id],
            )?;
            conn.execute(
                "UPDATE audit_log SET actor_id = NULL WHERE actor_id = ?",
                [user_id],
            )?;
            repo.delete(user_id)
        })
    }
}

fn validate_username(raw: &str) -> AppResult<String> {
    let username = raw.trim();
    if username.len() < 3 || username.len() > 32 {
        return Err(AppError::validation(
            "The username must be between 3 and 32 characters",
        ));
    }
    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
    {
        return Err(AppError::validation(
            "The username may only contain letters, numbers, dots, dashes and underscores",
        ));
    }
    Ok(username.to_string())
}

fn validate_display_name(raw: &str) -> AppResult<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(AppError::validation("Enter a display name"));
    }
    if name.chars().count() > 80 {
        return Err(AppError::validation(
            "The display name must be 80 characters or fewer",
        ));
    }
    Ok(name.to_string())
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.len() < MIN_PASSWORD_LEN {
        return Err(AppError::validation(format!(
            "The password must be at least {} characters",
            MIN_PASSWORD_LEN
        )));
    }
    if password.len() > MAX_PASSWORD_LEN {
        return Err(AppError::validation(format!(
            "The password must be {} characters or fewer",
            MAX_PASSWORD_LEN
        )));
    }
    Ok(())
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::new("HASH_ERROR", format!("Could not hash password: {error}")))
}

fn verify_password(hash: &str, password: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (PathBuf, Db) {
        let dir =
            std::env::temp_dir().join(format!("sfs-auth-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let db = Db::open(&dir).unwrap();
        (dir, db)
    }

    fn admin_input() -> CreateUserInput {
        CreateUserInput {
            username: "admin".into(),
            password: "correct horse".into(),
            display_name: "School Admin".into(),
        }
    }

    #[test]
    fn first_user_can_be_created_without_auth_and_becomes_the_admin() {
        let (dir, db) = temp_db("bootstrap");
        assert!(!AuthService::has_users(&db).unwrap());

        let user = AuthService::create_user(&db, &admin_input(), false).unwrap();
        assert_eq!(user.username, "admin");
        assert!(AuthService::has_users(&db).unwrap());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn second_user_requires_auth() {
        let (dir, db) = temp_db("second");
        AuthService::create_user(&db, &admin_input(), false).unwrap();

        let input = CreateUserInput {
            username: "clerk".into(),
            password: "another password".into(),
            display_name: "Clerk".into(),
        };
        let error = AuthService::create_user(&db, &input, false).unwrap_err();
        assert_eq!(error.code, "UNAUTHORIZED");

        AuthService::create_user(&db, &input, true).unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn login_accepts_the_right_password_case_insensitively_by_username() {
        let (dir, db) = temp_db("login");
        AuthService::create_user(&db, &admin_input(), false).unwrap();

        let user = AuthService::login(&db, "ADMIN", "correct horse").unwrap();
        assert_eq!(user.username, "admin");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn login_rejects_a_wrong_password() {
        let (dir, db) = temp_db("wrongpw");
        AuthService::create_user(&db, &admin_input(), false).unwrap();

        let error = AuthService::login(&db, "admin", "nope").unwrap_err();
        assert_eq!(error.code, "INVALID_CREDENTIALS");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn duplicate_usernames_are_rejected() {
        let (dir, db) = temp_db("dupe");
        AuthService::create_user(&db, &admin_input(), false).unwrap();

        let error = AuthService::create_user(&db, &admin_input(), true).unwrap_err();
        assert_eq!(error.code, "CONFLICT");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn short_passwords_are_rejected_and_passwords_can_be_changed() {
        let (dir, db) = temp_db("password");
        let user = AuthService::create_user(&db, &admin_input(), false).unwrap();

        let weak = CreateUserInput {
            username: "weak".into(),
            password: "short".into(),
            display_name: "Weak".into(),
        };
        assert_eq!(
            AuthService::create_user(&db, &weak, true).unwrap_err().code,
            "VALIDATION"
        );

        AuthService::change_password(
            &db,
            &user.id,
            &ChangePasswordInput {
                current_password: "correct horse".into(),
                new_password: "a brand new password".into(),
            },
        )
        .unwrap();

        assert!(AuthService::login(&db, "admin", "a brand new password").is_ok());
        assert!(AuthService::login(&db, "admin", "correct horse").is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn deactivated_users_cannot_log_in() {
        let (dir, db) = temp_db("deactivated");
        let user = AuthService::create_user(&db, &admin_input(), false).unwrap();
        AuthService::set_active(&db, &user.id, false).unwrap();

        let error = AuthService::login(&db, "admin", "correct horse").unwrap_err();
        assert_eq!(error.code, "FORBIDDEN");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn users_can_be_renamed_and_deleted() {
        let (dir, db) = temp_db("rename");
        let user = AuthService::create_user(&db, &admin_input(), false).unwrap();

        let updated =
            AuthService::update_user(&db, &user.id, "head.teacher", "Head Teacher").unwrap();
        assert_eq!(updated.username, "head.teacher");
        assert_eq!(updated.display_name, "Head Teacher");
        assert!(AuthService::login(&db, "head.teacher", "correct horse").is_ok());
        assert!(AuthService::login(&db, "admin", "correct horse").is_err());

        let clerk = AuthService::create_user(
            &db,
            &CreateUserInput {
                username: "clerk".into(),
                password: "another password".into(),
                display_name: "Clerk".into(),
            },
            true,
        )
        .unwrap();

        let error = AuthService::update_user(&db, &clerk.id, "head.teacher", "Clerk").unwrap_err();
        assert_eq!(error.code, "CONFLICT");

        AuthService::delete_user(&db, &clerk.id).unwrap();
        assert_eq!(AuthService::list_users(&db).unwrap().len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
