use crate::domain::auth::{ChangePasswordInput, CreateUserInput, User, UserRole};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::UserRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::user_repository::SqliteUserRepository;
use argon2::password_hash::{
    rand_core::{OsRng, RngCore},
    PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use rusqlite::OptionalExtension;
use serde_json::json;
use sha2::{Digest, Sha256};
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
                role: if is_first_user {
                    UserRole::SchoolAdministrator
                } else {
                    input.role
                },
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

        db.transaction(|conn| {
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

            repo.update_password(user_id, &hash_password(&new_password)?)?;
            conn.execute(
                "DELETE FROM remembered_sessions WHERE user_id = ?",
                [user_id],
            )?;
            Ok(())
        })
    }

    pub fn create_administrator_recovery_code(db: &Db, user_id: &str) -> AppResult<String> {
        let code = generate_recovery_code();
        let code_hash = hash_recovery_code(&code);

        db.transaction(|conn| {
            let user = SqliteUserRepository::new(conn)
                .find_by_id(user_id)?
                .ok_or_else(AppError::unauthorized)?;
            if !user.is_active || user.role != UserRole::SchoolAdministrator {
                return Err(AppError::forbidden(
                    "Only an active School Administrator can manage recovery",
                ));
            }
            ensure_sole_active_administrator(conn)?;
            let now = crate::application::now_iso();
            conn.execute(
                "INSERT INTO administrator_recovery_codes (user_id, code_hash, updated_at) \
                 VALUES (?, ?, ?) \
                 ON CONFLICT(user_id) DO UPDATE SET code_hash = excluded.code_hash, \
                 updated_at = excluded.updated_at",
                rusqlite::params![user.id, code_hash, now],
            )?;
            crate::application::audit_service::AuditService::record(
                conn,
                Some(&user.id),
                "administrator_recovery",
                &user.id,
                "code_rotated",
                json!({}),
                &now,
            )?;
            Ok(())
        })?;

        Ok(code)
    }

    pub fn administrator_recovery_configured(db: &Db, user_id: &str) -> AppResult<bool> {
        db.with_conn(|conn| {
            let configured = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM administrator_recovery_codes WHERE user_id = ?)",
                [user_id],
                |row| row.get(0),
            )?;
            Ok(configured)
        })
    }

    pub fn recover_administrator(
        db: &Db,
        username: &str,
        recovery_code: &str,
        new_password: &str,
    ) -> AppResult<String> {
        validate_password(new_password)?;
        let username = username.trim();
        if username.is_empty() || username.len() > 128 {
            return Err(AppError::validation("Enter your administrator username"));
        }

        let username_hash = hash_session_token(&username.to_ascii_lowercase());
        let supplied_code_hash = hash_recovery_code(recovery_code);
        let next_code = generate_recovery_code();
        let next_code_hash = hash_recovery_code(&next_code);
        let now_epoch = chrono::Utc::now().timestamp();
        let result = db.transaction(|conn| {
            let locked_until: Option<i64> = conn
                .query_row(
                    "SELECT locked_until FROM administrator_recovery_attempts \
                     WHERE username_hash = ?",
                    [&username_hash],
                    |row| row.get(0),
                )
                .optional()?;
            if let Some(locked_until) = locked_until.filter(|until| *until > now_epoch) {
                return Ok(Err(AppError::new(
                    "RATE_LIMITED",
                    format!(
                        "Too many recovery attempts. Try again in {} minute{}.",
                        ((locked_until - now_epoch + 59) / 60).max(1),
                        if locked_until - now_epoch > 60 { "s" } else { "" }
                    ),
                )));
            }

            let user = SqliteUserRepository::new(conn).find_by_username(username)?;
            let recovery_hash = if let Some(user) = &user {
                conn.query_row(
                    "SELECT code_hash FROM administrator_recovery_codes WHERE user_id = ?",
                    [&user.id],
                    |row| row.get::<_, String>(0),
                )
                .optional()?
            } else {
                None
            };
            let valid_user = user.as_ref().is_some_and(|user| {
                user.is_active && user.role == UserRole::SchoolAdministrator
            });
            let valid_code = recovery_hash
                .as_deref()
                .is_some_and(|hash| hash == supplied_code_hash);
            let active_administrators: i64 = conn.query_row(
                "SELECT COUNT(*) FROM users WHERE role = 'SCHOOL_ADMINISTRATOR' \
                 AND is_active = 1",
                [],
                |row| row.get(0),
            )?;
            let sole_admin = active_administrators == 1;

            if !valid_user || !valid_code || !sole_admin {
                let failed_attempts: i64 = conn.query_row(
                    "SELECT failed_attempts FROM administrator_recovery_attempts \
                     WHERE username_hash = ?",
                    [&username_hash],
                    |row| row.get(0),
                ).optional()?.unwrap_or(0);
                let next_attempts = failed_attempts + 1;
                let locked_until = if next_attempts >= 5 {
                    now_epoch + 15 * 60
                } else {
                    0
                };
                conn.execute(
                    "INSERT INTO administrator_recovery_attempts \
                     (username_hash, failed_attempts, locked_until) VALUES (?, ?, ?) \
                     ON CONFLICT(username_hash) DO UPDATE SET \
                     failed_attempts = excluded.failed_attempts, \
                     locked_until = excluded.locked_until",
                    rusqlite::params![username_hash, next_attempts, locked_until],
                )?;
                let entity_id = user
                    .as_ref()
                    .map(|user| user.id.clone())
                    .unwrap_or_else(|| username_hash.clone());
                crate::application::audit_service::AuditService::record(
                    conn,
                    None,
                    "administrator_recovery",
                    &entity_id,
                    "attempt_failed",
                    json!({}),
                    &crate::application::now_iso(),
                )?;
                let error = if next_attempts >= 5 {
                    AppError::new(
                        "RATE_LIMITED",
                        "Too many recovery attempts. Try again in 15 minutes.",
                    )
                } else {
                    AppError::new(
                        "RECOVERY_FAILED",
                        "The recovery details are incorrect or recovery is unavailable for this account",
                    )
                };
                return Ok(Err(error));
            }

            let Some(user) = user else {
                return Ok(Err(AppError::new(
                    "RECOVERY_FAILED",
                    "The recovery details are incorrect or recovery is unavailable for this account",
                )));
            };
            let new_password_hash = hash_password(new_password)?;
            let repo = SqliteUserRepository::new(conn);
            repo.update_password(&user.id, &new_password_hash)?;
            conn.execute(
                "DELETE FROM remembered_sessions WHERE user_id = ?",
                [&user.id],
            )?;
            conn.execute(
                "UPDATE administrator_recovery_codes SET code_hash = ?, updated_at = ? \
                 WHERE user_id = ?",
                rusqlite::params![
                    next_code_hash,
                    crate::application::now_iso(),
                    user.id
                ],
            )?;
            conn.execute(
                "DELETE FROM administrator_recovery_attempts WHERE username_hash = ?",
                [&username_hash],
            )?;
            crate::application::audit_service::AuditService::record(
                conn,
                Some(&user.id),
                "administrator_recovery",
                &user.id,
                "password_recovered",
                json!({}),
                &crate::application::now_iso(),
            )?;
            Ok(Ok(()))
        })?;

        result.map(|()| next_code)
    }

    pub fn create_remembered_session(db: &Db, user_id: &str) -> AppResult<String> {
        let token = Uuid::new_v4().to_string();
        let token_hash = hash_session_token(&token);
        db.transaction(|conn| {
            let user = SqliteUserRepository::new(conn)
                .find_by_id(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))?;
            if !user.is_active {
                return Err(AppError::forbidden("This account has been deactivated"));
            }
            conn.execute("DELETE FROM remembered_sessions", [])?;
            conn.execute(
                "INSERT INTO remembered_sessions (token_hash, user_id, created_at) \
                 VALUES (?, ?, ?)",
                rusqlite::params![token_hash, user_id, crate::application::now_iso()],
            )?;
            Ok(())
        })?;
        Ok(token)
    }

    pub fn restore_remembered_session(db: &Db, token: &str) -> AppResult<Option<User>> {
        let token_hash = hash_session_token(token);
        db.transaction(|conn| {
            let user_id = conn
                .query_row(
                    "SELECT user_id FROM remembered_sessions WHERE token_hash = ?",
                    [&token_hash],
                    |row| row.get::<_, String>(0),
                )
                .optional()?;
            let Some(user_id) = user_id else {
                return Ok(None);
            };
            let user = SqliteUserRepository::new(conn).find_by_id(&user_id)?;
            let Some(user) = user.filter(|user| user.is_active) else {
                conn.execute(
                    "DELETE FROM remembered_sessions WHERE token_hash = ?",
                    [&token_hash],
                )?;
                return Ok(None);
            };
            conn.execute(
                "UPDATE users SET last_login_at = ? WHERE id = ?",
                rusqlite::params![crate::application::now_iso(), user.id],
            )?;
            Ok(Some(user))
        })
    }

    pub fn revoke_remembered_session(db: &Db, token: &str) -> AppResult<()> {
        let token_hash = hash_session_token(token);
        db.with_conn(|conn| {
            conn.execute(
                "DELETE FROM remembered_sessions WHERE token_hash = ?",
                [&token_hash],
            )?;
            Ok(())
        })
    }

    pub fn set_active(db: &Db, user_id: &str, is_active: bool) -> AppResult<()> {
        db.transaction(|conn| {
            let repo = SqliteUserRepository::new(conn);
            let user = repo
                .find_by_id(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))?;
            if !is_active && user.role == UserRole::SchoolAdministrator {
                ensure_another_active_administrator(conn, user_id)?;
            }
            if !is_active {
                conn.execute(
                    "DELETE FROM remembered_sessions WHERE user_id = ?",
                    [user_id],
                )?;
            }
            repo.set_active(user_id, is_active)
        })
    }

    /// Changes a user's username and display name, returning the updated user.
    pub fn update_user(
        db: &Db,
        user_id: &str,
        username: &str,
        display_name: &str,
        role: UserRole,
    ) -> AppResult<User> {
        let username = validate_username(username)?;
        let display_name = validate_display_name(display_name)?;

        db.transaction(|conn| {
            let repo = SqliteUserRepository::new(conn);
            let existing = repo
                .find_by_id(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))?;
            if existing.is_active
                && existing.role == UserRole::SchoolAdministrator
                && role != UserRole::SchoolAdministrator
            {
                ensure_another_active_administrator(conn, user_id)?;
            }
            if let Some(existing) = repo.find_by_username(&username)? {
                if existing.id != user_id {
                    return Err(AppError::conflict(format!(
                        "The username \"{}\" is already taken",
                        username
                    )));
                }
            }
            repo.update_profile(user_id, &username, &display_name, role)
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
            let user = repo
                .find_by_id(user_id)?
                .ok_or_else(|| AppError::not_found("User", user_id))?;
            if user.is_active && user.role == UserRole::SchoolAdministrator {
                ensure_another_active_administrator(conn, user_id)?;
            }
            conn.execute(
                "UPDATE payments SET created_by = NULL WHERE created_by = ?",
                [user_id],
            )?;
            conn.execute(
                "UPDATE audit_log SET actor_id = NULL WHERE actor_id = ?",
                [user_id],
            )?;
            conn.execute(
                "DELETE FROM remembered_sessions WHERE user_id = ?",
                [user_id],
            )?;
            repo.delete(user_id)
        })
    }
}

fn hash_session_token(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn generate_recovery_code() -> String {
    let mut bytes = [0; 16];
    OsRng.fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|byte| format!("{byte:02X}")).collect();
    format!(
        "SFS-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..16],
        &hex[16..24],
        &hex[24..32]
    )
}

fn hash_recovery_code(code: &str) -> String {
    let normalized: String = code
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_uppercase())
        .collect();
    hash_session_token(&normalized)
}

fn ensure_sole_active_administrator(conn: &rusqlite::Connection) -> AppResult<()> {
    let active_administrators: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'SCHOOL_ADMINISTRATOR' AND is_active = 1",
        [],
        |row| row.get(0),
    )?;
    if active_administrators != 1 {
        return Err(AppError::conflict(
            "Recovery codes are available only when there is exactly one active School Administrator",
        ));
    }
    Ok(())
}

fn ensure_another_active_administrator(
    conn: &rusqlite::Connection,
    excluded_user_id: &str,
) -> AppResult<()> {
    let active_administrators: i64 = conn.query_row(
        "SELECT COUNT(*) FROM users WHERE role = 'SCHOOL_ADMINISTRATOR' \
         AND is_active = 1 AND id <> ?",
        [excluded_user_id],
        |row| row.get(0),
    )?;
    if active_administrators == 0 {
        return Err(AppError::conflict(
            "At least one active School Administrator account must remain",
        ));
    }
    Ok(())
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
            role: UserRole::SchoolAdministrator,
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
            role: UserRole::Bursar,
        };
        let error = AuthService::create_user(&db, &input, false).unwrap_err();
        assert_eq!(error.code, "UNAUTHORIZED");

        let user = AuthService::create_user(&db, &input, true).unwrap();
        assert_eq!(user.role, UserRole::Bursar);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn first_user_is_always_a_school_administrator() {
        let (dir, db) = temp_db("first_role");
        let input = CreateUserInput {
            role: UserRole::Bursar,
            ..admin_input()
        };

        let user = AuthService::create_user(&db, &input, false).unwrap();

        assert_eq!(user.role, UserRole::SchoolAdministrator);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn remembered_session_restores_until_password_change_or_logout() {
        let (dir, db) = temp_db("remembered_session");
        let admin = AuthService::create_user(&db, &admin_input(), false).unwrap();
        let token = AuthService::create_remembered_session(&db, &admin.id).unwrap();

        assert_eq!(
            AuthService::restore_remembered_session(&db, &token)
                .unwrap()
                .unwrap()
                .id,
            admin.id
        );
        AuthService::change_password(
            &db,
            &admin.id,
            &ChangePasswordInput {
                current_password: "correct horse".into(),
                new_password: "another correct horse".into(),
            },
        )
        .unwrap();
        assert!(AuthService::restore_remembered_session(&db, &token)
            .unwrap()
            .is_none());

        let next_token = AuthService::create_remembered_session(&db, &admin.id).unwrap();
        AuthService::revoke_remembered_session(&db, &next_token).unwrap();
        assert!(AuthService::restore_remembered_session(&db, &next_token)
            .unwrap()
            .is_none());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn deactivating_a_user_revokes_their_remembered_session() {
        let (dir, db) = temp_db("remembered_deactivation");
        AuthService::create_user(&db, &admin_input(), false).unwrap();
        let bursar = AuthService::create_user(
            &db,
            &CreateUserInput {
                username: "clerk".into(),
                password: "another password".into(),
                display_name: "Clerk".into(),
                role: UserRole::Bursar,
            },
            true,
        )
        .unwrap();
        let token = AuthService::create_remembered_session(&db, &bursar.id).unwrap();

        AuthService::set_active(&db, &bursar.id, false).unwrap();

        assert!(AuthService::restore_remembered_session(&db, &token)
            .unwrap()
            .is_none());
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
            role: UserRole::Bursar,
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
        AuthService::create_user(&db, &admin_input(), false).unwrap();
        let user = AuthService::create_user(
            &db,
            &CreateUserInput {
                username: "clerk".into(),
                password: "another password".into(),
                display_name: "Clerk".into(),
                role: UserRole::Bursar,
            },
            true,
        )
        .unwrap();
        AuthService::set_active(&db, &user.id, false).unwrap();

        let error = AuthService::login(&db, "clerk", "another password").unwrap_err();
        assert_eq!(error.code, "FORBIDDEN");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn last_active_administrator_cannot_be_deactivated_demoted_or_deleted() {
        let (dir, db) = temp_db("last_admin");
        let admin = AuthService::create_user(&db, &admin_input(), false).unwrap();

        assert_eq!(
            AuthService::set_active(&db, &admin.id, false)
                .unwrap_err()
                .code,
            "CONFLICT"
        );
        assert_eq!(
            AuthService::update_user(
                &db,
                &admin.id,
                &admin.username,
                &admin.display_name,
                UserRole::Bursar,
            )
            .unwrap_err()
            .code,
            "CONFLICT"
        );
        assert_eq!(
            AuthService::delete_user(&db, &admin.id).unwrap_err().code,
            "CONFLICT"
        );

        let remaining = AuthService::list_users(&db).unwrap();
        assert_eq!(remaining.len(), 1);
        assert!(remaining[0].is_active);
        assert_eq!(remaining[0].role, UserRole::SchoolAdministrator);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn users_can_be_renamed_and_deleted() {
        let (dir, db) = temp_db("rename");
        let user = AuthService::create_user(&db, &admin_input(), false).unwrap();

        let updated = AuthService::update_user(
            &db,
            &user.id,
            "head.teacher",
            "Head Teacher",
            UserRole::SchoolAdministrator,
        )
        .unwrap();
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
                role: UserRole::Bursar,
            },
            true,
        )
        .unwrap();

        let error =
            AuthService::update_user(&db, &clerk.id, "head.teacher", "Clerk", UserRole::Bursar)
                .unwrap_err();
        assert_eq!(error.code, "CONFLICT");

        AuthService::delete_user(&db, &clerk.id).unwrap();
        assert_eq!(AuthService::list_users(&db).unwrap().len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn administrator_recovery_rotates_the_one_time_code_and_revokes_sessions() {
        let (dir, db) = temp_db("administrator_recovery");
        let admin = AuthService::create_user(&db, &admin_input(), false).unwrap();
        let remembered_token = AuthService::create_remembered_session(&db, &admin.id).unwrap();
        let initial_code = AuthService::create_administrator_recovery_code(&db, &admin.id).unwrap();
        let stored_hash: String = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT code_hash FROM administrator_recovery_codes WHERE user_id = ?",
                    [&admin.id],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_ne!(stored_hash, initial_code);

        let replacement_code = AuthService::recover_administrator(
            &db,
            "ADMIN",
            &initial_code.to_ascii_lowercase(),
            "a newly chosen password",
        )
        .unwrap();
        assert_ne!(replacement_code, initial_code);
        assert!(AuthService::login(&db, "admin", "correct horse").is_err());
        assert!(AuthService::login(&db, "admin", "a newly chosen password").is_ok());
        assert!(
            AuthService::restore_remembered_session(&db, &remembered_token)
                .unwrap()
                .is_none()
        );
        assert_eq!(
            AuthService::recover_administrator(
                &db,
                "admin",
                &initial_code,
                "another new password",
            )
            .unwrap_err()
            .code,
            "RECOVERY_FAILED"
        );
        assert!(AuthService::recover_administrator(
            &db,
            "admin",
            &replacement_code,
            "another new password",
        )
        .is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn administrator_recovery_throttles_and_only_works_with_one_active_admin() {
        let (dir, db) = temp_db("administrator_recovery_throttle");
        let admin = AuthService::create_user(&db, &admin_input(), false).unwrap();
        let code = AuthService::create_administrator_recovery_code(&db, &admin.id).unwrap();
        for attempt in 0..5 {
            let error = AuthService::recover_administrator(
                &db,
                &admin.username,
                "incorrect recovery code",
                "a newly chosen password",
            )
            .unwrap_err();
            if attempt < 4 {
                assert_eq!(error.code, "RECOVERY_FAILED");
            } else {
                assert_eq!(error.code, "RATE_LIMITED");
            }
        }
        assert_eq!(
            AuthService::recover_administrator(
                &db,
                &admin.username,
                &code,
                "a newly chosen password",
            )
            .unwrap_err()
            .code,
            "RATE_LIMITED"
        );

        let another_admin = AuthService::create_user(
            &db,
            &CreateUserInput {
                username: "admin2".into(),
                password: "another correct password".into(),
                display_name: "Another Admin".into(),
                role: UserRole::SchoolAdministrator,
            },
            true,
        )
        .unwrap();
        assert_eq!(
            AuthService::create_administrator_recovery_code(&db, &another_admin.id)
                .unwrap_err()
                .code,
            "CONFLICT"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn recovery_is_not_available_when_multiple_active_administrators_exist() {
        let (dir, db) = temp_db("administrator_recovery_multiple_admins");
        let admin = AuthService::create_user(&db, &admin_input(), false).unwrap();
        let code = AuthService::create_administrator_recovery_code(&db, &admin.id).unwrap();
        AuthService::create_user(
            &db,
            &CreateUserInput {
                username: "admin2".into(),
                password: "another correct password".into(),
                display_name: "Another Admin".into(),
                role: UserRole::SchoolAdministrator,
            },
            true,
        )
        .unwrap();

        assert_eq!(
            AuthService::recover_administrator(
                &db,
                &admin.username,
                &code,
                "a newly chosen password",
            )
            .unwrap_err()
            .code,
            "RECOVERY_FAILED"
        );
        assert!(AuthService::login(&db, &admin.username, "correct horse").is_ok());

        let _ = fs::remove_dir_all(&dir);
    }
}
