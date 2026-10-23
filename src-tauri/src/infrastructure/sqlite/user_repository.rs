use crate::domain::auth::User;
use crate::domain::errors::AppResult;
use crate::domain::ports::UserRepository;
use rusqlite::{params, Connection};

pub struct SqliteUserRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteUserRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const USER_COLUMNS: &str =
    "id, username, display_name, is_active, created_at, last_login_at";

fn map_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        username: row.get(1)?,
        display_name: row.get(2)?,
        is_active: row.get(3)?,
        created_at: row.get(4)?,
        last_login_at: row.get(5)?,
    })
}

impl UserRepository for SqliteUserRepository<'_> {
    fn create(&self, user: &User, password_hash: &str) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO users (id, username, password_hash, display_name, is_active) \
             VALUES (?,?,?,?,?)",
            params![
                user.id,
                user.username,
                password_hash,
                user.display_name,
                user.is_active,
            ],
        )?;
        Ok(())
    }

    fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM users WHERE username = ?1 COLLATE NOCASE",
            USER_COLUMNS
        ))?;
        let mut rows = stmt.query(params![username])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_user(row)?)),
            None => Ok(None),
        }
    }

    fn find_by_id(&self, user_id: &str) -> AppResult<Option<User>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM users WHERE id = ?1",
            USER_COLUMNS
        ))?;
        let mut rows = stmt.query(params![user_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_user(row)?)),
            None => Ok(None),
        }
    }

    fn password_hash(&self, user_id: &str) -> AppResult<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT password_hash FROM users WHERE id = ?")?;
        let mut rows = stmt.query(params![user_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    fn list(&self) -> AppResult<Vec<User>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM users ORDER BY display_name",
            USER_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_user)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn count(&self) -> AppResult<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?)
    }

    fn update_password(&self, user_id: &str, password_hash: &str) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE users SET password_hash = ? WHERE id = ?",
            params![password_hash, user_id],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "User", user_id,
            ));
        }
        Ok(())
    }

    fn update_profile(
        &self,
        user_id: &str,
        username: &str,
        display_name: &str,
    ) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE users SET username = ?, display_name = ? WHERE id = ?",
            params![username, display_name, user_id],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "User", user_id,
            ));
        }
        Ok(())
    }

    fn set_active(&self, user_id: &str, is_active: bool) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE users SET is_active = ? WHERE id = ?",
            params![is_active, user_id],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "User", user_id,
            ));
        }
        Ok(())
    }

    fn record_login(&self, user_id: &str, at: &str) -> AppResult<()> {
        self.conn.execute(
            "UPDATE users SET last_login_at = ? WHERE id = ?",
            params![at, user_id],
        )?;
        Ok(())
    }

    fn delete(&self, user_id: &str) -> AppResult<()> {
        let changed = self
            .conn
            .execute("DELETE FROM users WHERE id = ?", params![user_id])?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "User", user_id,
            ));
        }
        Ok(())
    }
}
