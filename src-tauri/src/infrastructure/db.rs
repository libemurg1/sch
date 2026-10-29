use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::backup;
use crate::infrastructure::protected_key;
use argon2::password_hash::rand_core::{OsRng, RngCore};
use rusqlite::{Connection, OptionalExtension};
use std::fs;
use std::io::Read;
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
    key: Vec<u8>,
    startup_recovery_notice: Option<String>,
}

impl Db {
    /// Opens (creating if needed) the database inside `app_dir`, backing it up
    /// if migrations are pending and then applying any outstanding migrations.
    pub fn open(app_dir: &Path) -> AppResult<Self> {
        fs::create_dir_all(app_dir)?;
        let db_path = app_dir.join("school.db");
        let (key, conn, startup_recovery_notice) =
            match load_or_create_database_key(app_dir) {
                Ok(key) => {
                    match open_or_encrypt_database(&db_path, app_dir, &key) {
                        Ok(conn) => (key, conn, None),
                        Err(open_error) => {
                            match restore_from_latest_backup(&db_path, app_dir, &key) {
                        Ok((conn, backup_path, archive_path)) => (
                            key,
                            conn,
                            Some(format!(
                                "The database could not be opened, so it was recovered from {}. The damaged database was preserved at {}. Confirm recent records and create a new off-device backup.",
                                backup_path.display(),
                                archive_path.display()
                            )),
                        ),
                        Err(local_recovery_error) => {
                            let archive_path =
                                archive_for_portable_restore(&db_path, app_dir, false)?;
                            let conn = open_valid_database(&db_path, &key).map_err(|error| {
                                AppError::new(
                                    "DATABASE_RECOVERY_FAILED",
                                    format!(
                                        "The database could not be opened: {}. Local recovery failed: {}. A clean recovery database could not be initialized: {}",
                                        open_error, local_recovery_error, error
                                    ),
                                )
                            })?;
                            (
                                key,
                                conn,
                                Some(format!(
                                    "The database could not be opened and no usable local backup was available. The old database and local backups were preserved at {}. This is a new, empty recovery database. Before entering records, create an administrator account and restore an encrypted portable backup from System > Maintenance. If no portable backup exists, keep the archived files and contact support.",
                                    archive_path.display()
                                )),
                            )
                        }
                    }
                        }
                    }
                }
                Err(error)
                    if matches!(
                        error.code.as_str(),
                        "DB_KEY_MISSING" | "KEY_UNPROTECTION_FAILED"
                    ) =>
                {
                    let archive_path = archive_for_portable_restore(&db_path, app_dir, true)?;
                    let key = create_database_key(app_dir)?;
                    let conn = open_valid_database(&db_path, &key).map_err(|init_error| {
                    AppError::new(
                        "DATABASE_RECOVERY_FAILED",
                        format!(
                            "The database key could not be used and a clean recovery database could not be initialized: {}",
                            init_error
                        ),
                    )
                })?;
                    (
                    key,
                    conn,
                    Some(format!(
                        "The protected database key could not be used. The old database, key, and local backups were preserved at {}. This is a new, empty recovery database. Before entering records, create an administrator account and restore an encrypted portable backup from System > Maintenance. If no portable backup exists, keep the archived files and contact support.",
                        archive_path.display()
                    )),
                )
                }
                Err(error) => return Err(error),
            };

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
            backup::backup_database(&conn, app_dir, Some(&key))?;
        }

        apply_migrations(&conn)?;
        encrypt_legacy_backups(app_dir, &key, known_version)?;

        Ok(Self {
            conn: Mutex::new(conn),
            app_dir: app_dir.to_path_buf(),
            key,
            startup_recovery_notice,
        })
    }

    pub fn startup_recovery_notice(&self) -> Option<String> {
        self.startup_recovery_notice.clone()
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
        backup::backup_database(&conn, app_dir, Some(&self.key))
    }

    pub fn health_report(&self) -> AppResult<crate::domain::dto::DatabaseHealth> {
        let conn = self.lock()?;
        let schema_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        let integrity_check: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
        let foreign_keys_ok = conn
            .query_row("PRAGMA foreign_key_check", [], |row| {
                row.get::<_, String>(0)
            })
            .optional()?
            .is_none();

        let backups = backup::list_backups(&self.app_dir)?;
        let latest_backup_path = backups.first();
        let latest_backup_status = latest_backup_path.map(|path| {
            match backup::validate_backup(path, MIGRATIONS.len() as i32, Some(&self.key)) {
                Ok(()) => "Verified and readable".to_string(),
                Err(error) => format!("Verification failed: {}", error.message),
            }
        });

        Ok(crate::domain::dto::DatabaseHealth {
            checked_at: crate::application::now_iso(),
            schema_version,
            expected_schema_version: MIGRATIONS.len() as i32,
            integrity_check,
            foreign_keys_ok,
            backup_count: backups.len(),
            latest_backup_path: latest_backup_path.map(|path| path.to_string_lossy().into_owned()),
            latest_backup_status,
        })
    }

    pub fn export_backup(
        &self,
        backup_path: &Path,
        destination_directory: &Path,
        passphrase: &str,
    ) -> AppResult<PathBuf> {
        backup::export_backup(
            backup_path,
            destination_directory,
            &self.key,
            passphrase,
            MIGRATIONS.len() as i32,
        )
    }

    /// Restores the database from `backup_path`.
    ///
    /// The backup is copied into the open connection and any migrations the
    /// restored file is missing are applied afterwards, so a backup taken by an
    /// older build can still be restored.
    pub fn restore_from_backup(
        &self,
        backup_path: &Path,
        passphrase: Option<&str>,
    ) -> AppResult<()> {
        let current_path = self.app_dir.join("school.db");
        if backup_path.exists() && backup_path.canonicalize()? == current_path.canonicalize()? {
            return Err(AppError::new(
                "BACKUP_FAILED",
                "The live database cannot be restored as its own backup",
            ));
        }

        let max_schema_version = MIGRATIONS.len() as i32;
        let source = if backup_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("sfsbackup"))
            .unwrap_or(false)
        {
            let passphrase = passphrase.ok_or_else(|| {
                AppError::validation("Enter the passphrase used to encrypt this backup")
            })?;
            backup::open_portable_backup(backup_path, passphrase, max_schema_version)?
        } else {
            backup::open_local_backup(backup_path, &self.key, max_schema_version)?
        };

        let mut conn = self.lock()?;
        let safety_path = backup::backup_database(&conn, &self.app_dir, Some(&self.key))?;
        let safety_source = backup::open_local_backup(&safety_path, &self.key, max_schema_version)?;
        let restore_result = (|| -> AppResult<()> {
            backup::restore_from_connection(&source, &mut conn)?;
            apply_migrations(&conn)?;
            verify_restored_database(&conn)?;
            Ok(())
        })();

        if let Err(restore_error) = restore_result {
            let rollback_result = backup::restore_from_connection(&safety_source, &mut conn)
                .and_then(|_| apply_migrations(&conn))
                .and_then(|_| verify_restored_database(&conn));
            if let Err(rollback_error) = rollback_result {
                return Err(AppError::new(
                    "RESTORE_RECOVERY_FAILED",
                    format!(
                        "Restore failed: {}. The automatic rollback also failed: {}. Keep the application closed and contact support.",
                        restore_error, rollback_error
                    ),
                ));
            }
            return Err(AppError::new(
                "RESTORE_FAILED_ROLLED_BACK",
                format!(
                    "Restore failed and the original database was recovered: {}",
                    restore_error
                ),
            ));
        }

        Ok(())
    }
}

fn verify_restored_database(conn: &Connection) -> AppResult<()> {
    let integrity: String = conn.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::new(
            "RESTORE_FAILED",
            format!("The restored database failed its integrity check: {integrity}"),
        ));
    }
    let foreign_key_error: Option<(String, Option<i64>, String, i64)> = conn
        .query_row("PRAGMA foreign_key_check", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .optional()?;
    if foreign_key_error.is_some() {
        return Err(AppError::new(
            "RESTORE_FAILED",
            "The restored database contains broken record relationships",
        ));
    }
    Ok(())
}

fn load_or_create_database_key(app_dir: &Path) -> AppResult<Vec<u8>> {
    let key_path = app_dir.join("school.db.key");
    if key_path.exists() {
        return read_database_key(&key_path);
    }
    let database_path = app_dir.join("school.db");
    if database_path.exists() && fs::metadata(&database_path)?.len() > 0 {
        let mut file = fs::File::open(database_path)?;
        let mut header = [0; 16];
        if file.read_exact(&mut header).is_err() || &header != b"SQLite format 3\0" {
            return Err(AppError::new(
                "DB_KEY_MISSING",
                "The protected database key is missing. Do not reinstall or reset the Windows account; restore an encrypted portable backup with its passphrase.",
            ));
        }
    }

    create_database_key(app_dir)
}

fn create_database_key(app_dir: &Path) -> AppResult<Vec<u8>> {
    let key_path = app_dir.join("school.db.key");
    let mut key = vec![0; 32];
    OsRng.fill_bytes(&mut key);
    let protected = protected_key::protect_key(&key)?;
    let temporary_path = app_dir.join(format!("school.db.key.{}.tmp", uuid::Uuid::new_v4()));
    let write_result = (|| -> AppResult<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        use std::io::Write;
        file.write_all(&protected)?;
        file.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }

    if let Err(error) = fs::rename(&temporary_path, &key_path) {
        let _ = fs::remove_file(&temporary_path);
        if key_path.exists() {
            return read_database_key(&key_path);
        }
        return Err(AppError::from(error));
    }
    Ok(key)
}

fn read_database_key(key_path: &Path) -> AppResult<Vec<u8>> {
    let protected = fs::read(key_path)?;
    let key = protected_key::unprotect_key(&protected)?;
    if key.len() != 32 {
        return Err(AppError::new(
            "KEY_UNPROTECTION_FAILED",
            "The protected database key has an invalid length",
        ));
    }
    Ok(key)
}

fn open_or_encrypt_database(path: &Path, app_dir: &Path, key: &[u8]) -> AppResult<Connection> {
    if path.exists() && fs::metadata(path)?.len() > 0 {
        if let Ok(conn) = open_valid_database(path, key) {
            return Ok(conn);
        }

        let plaintext = Connection::open(path)?;
        let has_tables = has_user_tables(&plaintext)?;
        if !has_tables {
            return Err(AppError::new(
                "DB_ENCRYPTION_FAILED",
                "The database cannot be opened with its protected key or as a supported legacy database",
            ));
        }

        let _: i32 = plaintext.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        backup::backup_database(&plaintext, app_dir, None)?;
        drop(plaintext);
        backup::encrypt_legacy_database(path, key)?;
    }

    open_valid_database(path, key)
}

fn restore_from_latest_backup(
    database_path: &Path,
    app_dir: &Path,
    key: &[u8],
) -> AppResult<(Connection, PathBuf, PathBuf)> {
    let backups = backup::list_backups(app_dir)?;
    let mut last_error = None;
    for backup_path in backups {
        let source = match backup::open_local_backup(&backup_path, key, MIGRATIONS.len() as i32) {
            Ok(source) => source,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };
        let temporary_path =
            app_dir.join(format!("school.db.restore-{}.tmp", uuid::Uuid::new_v4()));
        let restore_result = (|| -> AppResult<()> {
            let mut replacement = Connection::open(&temporary_path)?;
            backup::set_database_key(&replacement, key)?;
            backup::restore_from_connection(&source, &mut replacement)?;
            apply_migrations(&replacement)?;
            verify_restored_database(&replacement)?;
            drop(replacement);
            Ok(())
        })();
        if let Err(error) = restore_result {
            let _ = fs::remove_file(&temporary_path);
            last_error = Some(error);
            continue;
        }
        drop(source);

        let archive_path =
            match archive_and_install_recovered_database(database_path, &temporary_path, app_dir) {
                Ok(path) => path,
                Err(error) => {
                    let _ = fs::remove_file(&temporary_path);
                    return Err(error);
                }
            };
        let conn = open_valid_database(database_path, key)?;
        return Ok((conn, backup_path, archive_path));
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::new("BACKUP_FAILED", "No local database backups are available")
    }))
}

fn archive_and_install_recovered_database(
    database_path: &Path,
    replacement_path: &Path,
    app_dir: &Path,
) -> AppResult<PathBuf> {
    let archive_path = app_dir.join(format!("school.db.damaged-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&archive_path)?;
    let mut moved_files = Vec::new();

    for suffix in ["", "-wal", "-shm"] {
        let mut source_name = database_path.as_os_str().to_os_string();
        source_name.push(suffix);
        let source = PathBuf::from(source_name);
        if !source.exists() {
            continue;
        }
        let file_name = source
            .file_name()
            .ok_or_else(|| AppError::validation("The database path has no file name"))?;
        let archived = archive_path.join(file_name);
        if let Err(error) = fs::rename(&source, &archived) {
            restore_archived_files(&moved_files).map_err(|restore_error| {
                AppError::new(
                    "DATABASE_RECOVERY_FAILED",
                    format!(
                        "Could not preserve the damaged database: {}. Reinstating its files also failed: {}",
                        error, restore_error
                    ),
                )
            })?;
            let _ = fs::remove_dir(&archive_path);
            return Err(AppError::from(error));
        }
        moved_files.push((source, archived));
    }

    if let Err(error) = fs::rename(replacement_path, database_path) {
        restore_archived_files(&moved_files).map_err(|restore_error| {
            AppError::new(
                "DATABASE_RECOVERY_FAILED",
                format!(
                    "Could not install the recovered database: {}. Reinstating the damaged database also failed: {}",
                    error, restore_error
                ),
            )
        })?;
        let _ = fs::remove_dir(&archive_path);
        return Err(AppError::from(error));
    }

    Ok(archive_path)
}

fn archive_for_portable_restore(
    database_path: &Path,
    app_dir: &Path,
    include_key: bool,
) -> AppResult<PathBuf> {
    let archive_path = app_dir.join(format!(
        "school.db.portable-restore-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir(&archive_path)?;
    let mut files = vec![database_path.to_path_buf()];
    for suffix in ["-wal", "-shm"] {
        let mut sidecar_name = database_path.as_os_str().to_os_string();
        sidecar_name.push(suffix);
        files.push(PathBuf::from(sidecar_name));
    }
    if include_key {
        files.push(app_dir.join("school.db.key"));
    }
    files.extend(backup::list_backups(app_dir)?);

    let mut moved_files = Vec::new();
    for source in files {
        if !source.exists() {
            continue;
        }
        let file_name = source
            .file_name()
            .ok_or_else(|| AppError::validation("The recovery path has no file name"))?;
        let archived = archive_path.join(file_name);
        if let Err(error) = fs::rename(&source, &archived) {
            restore_archived_files(&moved_files).map_err(|restore_error| {
                AppError::new(
                    "DATABASE_RECOVERY_FAILED",
                    format!(
                        "Could not preserve the old database files: {}. Reinstating them also failed: {}",
                        error, restore_error
                    ),
                )
            })?;
            let _ = fs::remove_dir(&archive_path);
            return Err(AppError::from(error));
        }
        moved_files.push((source, archived));
    }

    Ok(archive_path)
}

fn restore_archived_files(moved_files: &[(PathBuf, PathBuf)]) -> AppResult<()> {
    for (original, archived) in moved_files.iter().rev() {
        fs::rename(archived, original)?;
    }
    Ok(())
}

fn open_valid_database(path: &Path, key: &[u8]) -> AppResult<Connection> {
    let conn = backup::open_database(path, Some(key))?;
    let _: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    configure_connection(&conn)?;
    Ok(conn)
}

fn encrypt_legacy_backups(app_dir: &Path, key: &[u8], max_schema_version: i32) -> AppResult<()> {
    for path in backup::list_backups(app_dir)? {
        if backup::validate_backup(&path, max_schema_version, Some(key)).is_ok() {
            continue;
        }
        if backup::validate_backup(&path, max_schema_version, None).is_err() {
            continue;
        }

        backup::encrypt_legacy_database(&path, key)?;
        backup::validate_backup(&path, max_schema_version, Some(key))?;
    }
    Ok(())
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

        db.restore_from_backup(&backup_path, None).unwrap();

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
    fn restore_failure_rolls_back_to_the_pre_restore_database() {
        let dir = temp_dir("restore-rollback");
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            Ok(conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                 VALUES ('keep', 'Keep This Data', 1, '', 2026, 30)",
                [],
            )?)
        })
        .unwrap();

        let invalid_migration_backup = db.backup(&dir).unwrap();
        {
            let source = backup::open_database(&invalid_migration_backup, Some(&db.key)).unwrap();
            source
                .pragma_update(None, "user_version", MIGRATIONS.len() as i32 - 1)
                .unwrap();
        }

        let error = db
            .restore_from_backup(&invalid_migration_backup, None)
            .unwrap_err();
        assert_eq!(error.code, "RESTORE_FAILED_ROLLED_BACK");

        let record_exists: bool = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM classes WHERE id = 'keep')",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert!(record_exists);
        db.health_report().unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_backup_is_rejected_without_changing_live_data() {
        let dir = temp_dir("restore-invalid-backup");
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            Ok(conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                 VALUES ('keep', 'Keep This Data', 1, '', 2026, 30)",
                [],
            )?)
        })
        .unwrap();
        let invalid_backup = dir.join("school_backup_corrupted.db");
        fs::write(&invalid_backup, b"not a database").unwrap();

        assert!(db.restore_from_backup(&invalid_backup, None).is_err());
        let record_exists: bool = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM classes WHERE id = 'keep')",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert!(record_exists);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn health_report_verifies_database_and_latest_backup() {
        let dir = temp_dir("health-report");
        let db = Db::open(&dir).unwrap();
        let backup_path = db.backup(&dir).unwrap();

        let report = db.health_report().unwrap();
        assert_eq!(report.integrity_check, "ok");
        assert!(report.foreign_keys_ok);
        assert_eq!(report.backup_count, 1);
        assert_eq!(
            report.latest_backup_path.as_deref(),
            Some(backup_path.to_string_lossy().as_ref())
        );
        assert_eq!(
            report.latest_backup_status.as_deref(),
            Some("Verified and readable")
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn health_report_flags_a_corrupted_latest_backup() {
        let dir = temp_dir("health-invalid-backup");
        let db = Db::open(&dir).unwrap();
        db.backup(&dir).unwrap();
        let corrupted_backup = dir.join("school_backup_99999999_corrupted.db");
        fs::write(&corrupted_backup, b"not a database").unwrap();

        let report = db.health_report().unwrap();
        assert_eq!(report.backup_count, 2);
        assert_eq!(
            report.latest_backup_path.as_deref(),
            Some(corrupted_backup.to_string_lossy().as_ref())
        );
        assert!(report
            .latest_backup_status
            .as_deref()
            .is_some_and(|status| status.starts_with("Verification failed:")));

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

    #[test]
    fn new_database_file_is_encrypted_at_rest() {
        let dir = temp_dir("ciphertext");
        let db = Db::open(&dir).unwrap();
        drop(db);

        let database_bytes = fs::read(dir.join("school.db")).unwrap();
        assert!(!database_bytes.starts_with(b"SQLite format 3"));
        assert!(dir.join("school.db.key").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_live_database_recovers_from_a_verified_local_backup() {
        let dir = temp_dir("startup-recovery");
        let db_path = dir.join("school.db");
        let backup_path;
        {
            let db = Db::open(&dir).unwrap();
            db.transaction(|conn| {
                Ok(conn.execute(
                    "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                     VALUES ('recover-me', 'Recovered Class', 1, '', 2026, 30)",
                    [],
                )?)
            })
            .unwrap();
            backup_path = db.backup(&dir).unwrap();
        }
        fs::write(&db_path, b"damaged database bytes").unwrap();

        let recovered = Db::open(&dir).unwrap();
        let record_exists: bool = recovered
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT EXISTS(SELECT 1 FROM classes WHERE id = 'recover-me')",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert!(record_exists);
        assert!(recovered.startup_recovery_notice().is_some_and(
            |notice| notice.contains(backup_path.file_name().unwrap().to_str().unwrap())
        ));
        assert!(fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .any(|entry| entry.file_type().unwrap().is_dir()
                && entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("school.db.damaged-")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_key_preserves_the_old_database_and_opens_portable_restore_mode() {
        let dir = temp_dir("missing-db-key");
        let db_path = dir.join("school.db");
        {
            let db = Db::open(&dir).unwrap();
            db.transaction(|conn| {
                Ok(conn.execute(
                    "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                     VALUES ('keep', 'Keep This Data', 1, '', 2026, 30)",
                    [],
                )?)
            })
            .unwrap();
        }
        let database_bytes = fs::read(&db_path).unwrap();
        fs::remove_file(dir.join("school.db.key")).unwrap();

        let recovered = Db::open(&dir).unwrap();
        assert!(recovered
            .startup_recovery_notice()
            .is_some_and(|notice| notice.contains("new, empty recovery database")));
        assert!(dir.join("school.db.key").exists());
        let archived_database = fs::read_dir(&dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().unwrap().is_dir())
            .map(|entry| entry.path().join("school.db"))
            .find(|path| path.exists());
        assert!(archived_database.is_some_and(|path| fs::read(path).unwrap() == database_bytes));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn damaged_database_without_local_backup_opens_portable_restore_mode() {
        let dir = temp_dir("missing-local-backup");
        let db_path = dir.join("school.db");
        {
            let db = Db::open(&dir).unwrap();
            db.transaction(|conn| {
                Ok(conn.execute(
                    "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                     VALUES ('keep', 'Keep This Data', 1, '', 2026, 30)",
                    [],
                )?)
            })
            .unwrap();
        }
        fs::write(&db_path, b"damaged database bytes").unwrap();

        let recovered = Db::open(&dir).unwrap();
        assert!(recovered
            .startup_recovery_notice()
            .is_some_and(|notice| notice.contains("restore an encrypted portable backup")));
        let user_tables: i64 = recovered
            .with_conn(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(user_tables, 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_plaintext_database_and_snapshots_are_encrypted_on_open() {
        let dir = temp_dir("legacy-encryption");
        {
            let conn = Connection::open(dir.join("school.db")).unwrap();
            configure_connection(&conn).unwrap();
            apply_migrations(&conn).unwrap();
        }

        let db = Db::open(&dir).unwrap();
        let version: i32 = db
            .with_conn(|conn| Ok(conn.query_row("PRAGMA user_version", [], |row| row.get(0))?))
            .unwrap();
        assert_eq!(version, MIGRATIONS.len() as i32);

        let database_bytes = fs::read(dir.join("school.db")).unwrap();
        assert!(!database_bytes.starts_with(b"SQLite format 3"));

        let backups = backup::list_backups(&dir).unwrap();
        assert_eq!(backups.len(), 1);
        assert!(
            backup::validate_backup(&backups[0], MIGRATIONS.len() as i32, Some(&db.key)).is_ok()
        );

        drop(db);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn portable_export_restores_data_and_rejects_a_wrong_passphrase() {
        let dir = temp_dir("portable-export");
        let export_dir = dir.join("external");
        fs::create_dir_all(&export_dir).unwrap();
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            Ok(conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                 VALUES ('portable-class', 'Portable Class', 1, '', 2026, 30)",
                [],
            )?)
        })
        .unwrap();

        let snapshot = db.backup(&dir).unwrap();
        let export = db
            .export_backup(&snapshot, &export_dir, "portable backup passphrase")
            .unwrap();
        let exported_bytes = fs::read(&export).unwrap();
        assert!(!exported_bytes.starts_with(b"SQLite format 3"));
        assert!(!exported_bytes
            .windows(b"Portable Class".len())
            .any(|window| window == b"Portable Class"));

        assert!(db
            .restore_from_backup(&export, Some("incorrect portable passphrase"))
            .is_err());
        db.transaction(|conn| Ok(conn.execute("DELETE FROM classes", [])?))
            .unwrap();
        db.restore_from_backup(&export, Some("portable backup passphrase"))
            .unwrap();

        let restored: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM classes WHERE id = 'portable-class'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(restored, 1);
        drop(db);
        let _ = fs::remove_dir_all(&dir);
    }
}
