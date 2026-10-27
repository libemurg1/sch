use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::backup;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

include!(concat!(env!("OUT_DIR"), "/migrations_gen.rs"));

/// Owns the single SQLite connection used by the application.
///
/// The connection lives behind a mutex because `rusqlite::Connection` is `Send`
/// but not `Sync`. Repositories never hold the lock themselves - they borrow a
/// `&Connection` handed to them by [`Db::with_conn`] or [`Db::transaction`].
/// This is what makes nesting safe: a repository called from inside a
/// transaction reuses the transaction's connection instead of trying to lock
/// it again.
pub struct Db {
    conn: Mutex<Connection>,
    app_dir: PathBuf,
}

impl Db {
    /// Opens (creating if needed) the database inside `app_dir`, backing it up
    /// if migrations are pending and then applying any outstanding migrations.
    pub fn open(app_dir: &Path) -> AppResult<Self> {
        fs::create_dir_all(app_dir)?;
        let db_path = app_dir.join("school.db");

        let conn = Connection::open(&db_path)?;
        configure_connection(&conn)?;

        let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let known_version = MIGRATIONS.len() as i32;

        if current_version > known_version {
            return Err(AppError::new(
                "DB_TOO_NEW",
                format!(
                    "Database schema version {} is newer than this build understands ({}). \
                     Update the application or restore a backup.",
                    current_version, known_version
                ),
            ));
        }

        if current_version < known_version && has_user_tables(&conn)? {
            // Only a database that already holds data is worth a snapshot: a
            // fresh install has nothing to lose.
            backup::backup_database(&conn, app_dir)?;
        }

        apply_migrations(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
            app_dir: app_dir.to_path_buf(),
        })
    }

    /// Runs `f` with the shared connection outside of any transaction.
    pub fn with_conn<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.lock()?;
        f(&conn)
    }

    /// Runs `f` inside a single `BEGIN IMMEDIATE` transaction. The transaction
    /// is committed when `f` returns `Ok` and rolled back on `Err`.
    pub fn transaction<T>(&self, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
        let conn = self.lock()?;
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        match f(&conn) {
            Ok(value) => {
                conn.execute_batch("COMMIT;")?;
                Ok(value)
            }
            Err(error) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(error)
            }
        }
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|_| AppError::new("DB_POISONED", "The database lock was poisoned by a panic"))
    }

    /// Creates a consistent copy of the database using SQLite's online backup
    /// API and returns the path of the backup file.
    pub fn backup(&self, app_dir: &Path) -> AppResult<PathBuf> {
        let conn = self.lock()?;
        backup::backup_database(&conn, app_dir)
    }

    /// Restores the database from `backup_path`.
    ///
    /// The backup is copied into the open connection and any migrations the
    /// restored file is missing are applied afterwards, so a backup taken by an
    /// older build can still be restored.
    pub fn restore_from_backup(&self, backup_path: &Path) -> AppResult<()> {
        backup::validate_backup(backup_path, MIGRATIONS.len() as i32)?;
        let current_path = self.app_dir.join("school.db");
        if backup_path.canonicalize()? == current_path.canonicalize()? {
            return Err(AppError::new(
                "BACKUP_FAILED",
                "The live database cannot be restored as its own backup",
            ));
        }

        let mut conn = self.lock()?;
        backup::backup_database(&conn, &self.app_dir)?;
        backup::restore_database(&mut conn, backup_path)?;
        apply_migrations(&conn)?;
        Ok(())
    }
}

/// Applies `f` in a transaction, but only when the connection is not already
/// inside one. Nested calls therefore join the outer transaction instead of
/// failing with "cannot start a transaction within a transaction".
pub fn with_optional_transaction<T>(
    conn: &Connection,
    f: impl FnOnce() -> AppResult<T>,
) -> AppResult<T> {
    if !conn.is_autocommit() {
        return f();
    }

    conn.execute_batch("BEGIN IMMEDIATE;")?;
    match f() {
        Ok(value) => {
            conn.execute_batch("COMMIT;")?;
            Ok(value)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}

fn configure_connection(conn: &Connection) -> AppResult<()> {
    // `PRAGMA journal_mode` and `PRAGMA busy_timeout` return a row, so they
    // cannot be issued through `execute_batch`.
    let _: String = conn.query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))?;
    let _: i64 = conn.query_row("PRAGMA busy_timeout = 5000", [], |row| row.get(0))?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    Ok(())
}

/// Whether the database already holds tables, which means it holds data worth
/// backing up before a migration runs.
fn has_user_tables(conn: &Connection) -> AppResult<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

fn apply_migrations(conn: &Connection) -> AppResult<()> {
    let current_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    // Migrations that rebuild a table would otherwise make SQLite rewrite the
    // FOREIGN KEY clauses of every child table to point at the temporary name
    // (the rename happens while `foreign_keys` is ON). Disable enforcement for
    // the duration of the migrations and switch it back on afterwards; the
    // migration itself is responsible for copying every row.
    conn.execute_batch("PRAGMA foreign_keys = OFF;")?;

    let migrated = (|| -> AppResult<()> {
        for (index, (name, sql)) in MIGRATIONS.iter().enumerate() {
            let version = (index + 1) as i32;
            if version <= current_version {
                continue;
            }

            conn.execute_batch("BEGIN IMMEDIATE;")?;
            let applied = conn.execute_batch(sql).and_then(|_| {
                conn.pragma_update(None, "user_version", version)?;
                conn.execute_batch("COMMIT;")
            });

            if let Err(error) = applied {
                let _ = conn.execute_batch("ROLLBACK;");
                return Err(AppError::new(
                    "MIGRATION_FAILED",
                    format!("Migration {} failed: {}", name, error),
                ));
            }
        }
        Ok(())
    })();

    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    migrated
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sfs-test-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn migrations_are_applied_once_and_survive_reopen() {
        let dir = temp_dir("migrations");

        {
            let db = Db::open(&dir).unwrap();
            let version: i32 = db
                .with_conn(|conn| Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?))
                .unwrap();
            assert_eq!(version, MIGRATIONS.len() as i32);
        }

        // Re-opening must not re-apply migrations (this used to fail with
        // "table students already exists").
        {
            let db = Db::open(&dir).unwrap();
            let tables: i64 = db
                .with_conn(|conn| {
                    Ok(conn.query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'students'",
                        [],
                        |r| r.get(0),
                    )?)
                })
                .unwrap();
            assert_eq!(tables, 1);
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn fresh_install_does_not_create_a_backup() {
        let dir = temp_dir("fresh");
        let db = Db::open(&dir).unwrap();
        drop(db);

        let backups = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("school_backup_"))
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(backups, 0, "a fresh install has nothing worth backing up");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn install_seeds_grades_and_standard_classes() {
        let dir = temp_dir("seeds");
        let db = Db::open(&dir).unwrap();

        let (grades, classes): (i64, i64) = db
            .with_conn(|conn| {
                Ok((
                    conn.query_row("SELECT COUNT(*) FROM grades", [], |r| r.get(0))?,
                    conn.query_row("SELECT COUNT(*) FROM classes", [], |r| r.get(0))?,
                ))
            })
            .unwrap();
        assert_eq!(grades, 10);
        assert_eq!(classes, 10);

        let (level, name): (i32, String) = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT grade_level, name FROM classes WHERE name = 'ECD A'",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(level, -1);
        assert_eq!(name, "ECD A");

        let (level, stream): (i32, String) = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT grade_level, stream FROM classes WHERE name = 'Nursery'",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?)
            })
            .unwrap();
        assert_eq!(level, -2);
        assert_eq!(stream, "");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn restore_brings_back_the_backup_contents() {
        let dir = temp_dir("restore");
        let db = Db::open(&dir).unwrap();

        db.transaction(|conn| {
            Ok(conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) VALUES ('c1', 'Test Class', 1, '', 2026, 30)",
                [],
            )?)
        })
        .unwrap();

        let backup_path = db.backup(&dir).unwrap();

        db.transaction(|conn| Ok(conn.execute("DELETE FROM classes", [])?))
            .unwrap();

        let count: i64 = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT COUNT(*) FROM classes WHERE id = 'c1'", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(count, 0);

        db.restore_from_backup(&backup_path).unwrap();

        let count: i64 = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT COUNT(*) FROM classes WHERE id = 'c1'", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn transaction_rolls_back_on_error() {
        let dir = temp_dir("rollback");
        let db = Db::open(&dir).unwrap();

        let result: AppResult<()> = db.transaction(|conn| {
            conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) VALUES ('c1', 'Test Class', 1, '', 2026, 30)",
                [],
            )?;
            Err(AppError::new("BOOM", "forced failure"))
        });
        assert!(result.is_err());

        let count: i64 = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT COUNT(*) FROM classes WHERE id = 'c1'", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(count, 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn transaction_commits_on_success() {
        let dir = temp_dir("commit");
        let db = Db::open(&dir).unwrap();

        db.transaction(|conn| {
            Ok(conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) VALUES ('c1', 'Test Class', 1, '', 2026, 30)",
                [],
            )?)
        })
        .unwrap();

        let count: i64 = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT COUNT(*) FROM classes WHERE id = 'c1'", [], |r| {
                        r.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
