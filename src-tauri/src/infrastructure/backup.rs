use crate::domain::errors::{AppError, AppResult};
use rusqlite::backup::Backup;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MIN_EXPORT_PASSWORD_LEN: usize = 12;

/// Opens a database and applies the supplied SQLCipher raw key before reads.
pub fn open_database(path: &Path, key: Option<&[u8]>) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    if let Some(key) = key {
        set_database_key(&conn, key)?;
    }
    Ok(conn)
}

pub fn set_database_key(conn: &Connection, key: &[u8]) -> AppResult<()> {
    let key_hex: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
    conn.execute_batch(&format!("PRAGMA key = \"x'{key_hex}'\";"))?;
    Ok(())
}

fn set_passphrase_key(conn: &Connection, passphrase: &str) -> AppResult<()> {
    conn.execute_batch(&format!("PRAGMA key = {};", sql_string_literal(passphrase)))?;
    Ok(())
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

/// Rewrites an existing plaintext database through SQLCipher's supported
/// export flow, then atomically swaps in the encrypted file.
pub fn encrypt_legacy_database(path: &Path, key: &[u8]) -> AppResult<()> {
    let source = Connection::open(path)?;
    let schema_version: i32 = source.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let file_name = path
        .file_name()
        .ok_or_else(|| AppError::validation("The database path has no file name"))?
        .to_string_lossy();
    let encrypted_path = path.with_file_name(format!(
        "{file_name}.encrypted-{}.tmp",
        uuid::Uuid::new_v4()
    ));
    let key_hex: String = key.iter().map(|byte| format!("{byte:02x}")).collect();
    let attach = format!(
        "ATTACH DATABASE {} AS encrypted KEY \"x'{key_hex}'\";",
        sql_string_literal(&encrypted_path.to_string_lossy())
    );
    if let Err(error) = source.execute_batch(&attach) {
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(AppError::from(error));
    }

    let export_result = (|| -> rusqlite::Result<()> {
        source.execute_batch("SELECT sqlcipher_export('encrypted');")?;
        source.pragma_update(
            Some(rusqlite::DatabaseName::Attached("encrypted")),
            "user_version",
            schema_version,
        )
    })();
    let detach_result = source.execute_batch("DETACH DATABASE encrypted;");
    if let Err(error) = export_result {
        let _ = source.execute_batch("DETACH DATABASE encrypted;");
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(AppError::from(error));
    }
    if let Err(error) = detach_result {
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(AppError::from(error));
    }

    let (busy, _, _): (i64, i64, i64) =
        source.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
    if busy != 0 {
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(AppError::new(
            "DB_ENCRYPTION_FAILED",
            "The database could not be safely checkpointed before encryption",
        ));
    }
    drop(source);

    let encrypted = open_database(&encrypted_path, Some(key))?;
    let integrity: String = encrypted.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        drop(encrypted);
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(AppError::new(
            "DB_ENCRYPTION_FAILED",
            format!("The encrypted database failed its integrity check: {integrity}"),
        ));
    }
    drop(encrypted);

    remove_sidecar(path, "-wal")?;
    remove_sidecar(path, "-shm")?;
    if let Err(error) = replace_file(&encrypted_path, path) {
        let _ = std::fs::remove_file(&encrypted_path);
        return Err(error);
    }
    Ok(())
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let succeeded = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if succeeded == 0 {
        return Err(AppError::from(std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> AppResult<()> {
    std::fs::rename(source, destination)?;
    Ok(())
}

fn remove_sidecar(path: &Path, suffix: &str) -> AppResult<()> {
    let mut sidecar_name = path.as_os_str().to_os_string();
    sidecar_name.push(suffix);
    let sidecar = PathBuf::from(sidecar_name);
    match std::fs::remove_file(sidecar) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::from(error)),
    }
}

/// Confirms a supported, structurally intact database snapshot.
pub fn validate_backup(
    backup_path: &Path,
    max_schema_version: i32,
    key: Option<&[u8]>,
) -> AppResult<()> {
    if !backup_path.exists() {
        return Err(AppError::new(
            "BACKUP_FAILED",
            format!("Backup not found: {}", backup_path.display()),
        ));
    }

    let source = open_database(backup_path, key)?;
    validate_database(&source, max_schema_version)
}

fn validate_database(source: &Connection, max_schema_version: i32) -> AppResult<()> {
    let schema_version: i32 = source.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if schema_version == 0 || schema_version > max_schema_version {
        return Err(AppError::new(
            "BACKUP_FAILED",
            "This file is not a supported School Fees System database backup",
        ));
    }
    let integrity: String = source.query_row("PRAGMA quick_check", [], |row| row.get(0))?;
    if integrity != "ok" {
        return Err(AppError::new(
            "BACKUP_FAILED",
            format!("The backup failed SQLite's integrity check: {}", integrity),
        ));
    }
    let required_tables: i64 = source.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' \
         AND name IN ('users', 'students', 'invoices', 'payments', 'school_settings')",
        [],
        |row| row.get(0),
    )?;
    if required_tables != 5 {
        return Err(AppError::new(
            "BACKUP_FAILED",
            "This file is missing required School Fees System data",
        ));
    }
    Ok(())
}

/// Creates a consistent SQLCipher-protected snapshot using SQLite's online
/// backup API.
pub fn backup_database(
    conn: &Connection,
    app_dir: &Path,
    key: Option<&[u8]>,
) -> AppResult<PathBuf> {
    std::fs::create_dir_all(app_dir)?;

    let backup_path = app_dir.join(format!(
        "school_backup_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S_%3f")
    ));

    let mut destination = Connection::open(&backup_path)?;
    if let Some(key) = key {
        set_database_key(&destination, key)?;
    }
    let backup = Backup::new(conn, &mut destination)?;
    backup.run_to_completion(128, Duration::from_millis(10), None)?;

    Ok(backup_path)
}

/// Lists backups stored in `app_dir`, newest first.
pub fn list_backups(app_dir: &Path) -> AppResult<Vec<PathBuf>> {
    if !app_dir.exists() {
        return Ok(Vec::new());
    }

    let mut backups: Vec<PathBuf> = std::fs::read_dir(app_dir)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("school_backup_") && name.ends_with(".db"))
                .unwrap_or(false)
        })
        .collect();

    backups.sort_by(|left, right| right.cmp(left));
    Ok(backups)
}

/// Opens and validates a local snapshot, including legacy plaintext snapshots
/// created before SQLCipher was enabled.
pub fn open_local_backup(
    backup_path: &Path,
    key: &[u8],
    max_schema_version: i32,
) -> AppResult<Connection> {
    let source = match open_database(backup_path, Some(key)) {
        Ok(source) => match validate_database(&source, max_schema_version) {
            Ok(()) => source,
            Err(_) => open_legacy_backup(backup_path, max_schema_version)?,
        },
        Err(_) => open_legacy_backup(backup_path, max_schema_version)?,
    };

    Ok(source)
}

fn open_legacy_backup(path: &Path, max_schema_version: i32) -> AppResult<Connection> {
    let source = open_database(path, None)?;
    validate_database(&source, max_schema_version)?;
    Ok(source)
}

/// Encrypts an application snapshot with a passphrase into a portable
/// SQLCipher database. No plaintext database file is written to disk.
pub fn export_backup(
    backup_path: &Path,
    destination_directory: &Path,
    database_key: &[u8],
    passphrase: &str,
    max_schema_version: i32,
) -> AppResult<PathBuf> {
    validate_passphrase(passphrase)?;
    if !backup_path.exists() {
        return Err(AppError::new(
            "BACKUP_FAILED",
            format!("Backup not found: {}", backup_path.display()),
        ));
    }
    if !destination_directory.is_dir() {
        return Err(AppError::validation(
            "Choose an existing destination folder for the backup",
        ));
    }

    let source = open_database(backup_path, Some(database_key))?;
    validate_database(&source, max_schema_version)?;

    let file_name = backup_path
        .file_stem()
        .ok_or_else(|| AppError::validation("The backup path has no file name"))?;
    let destination = destination_directory
        .join(file_name)
        .with_extension("sfsbackup");
    let reservation = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&destination)
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::conflict(format!(
                    "A backup with this name already exists at {}",
                    destination.display()
                ))
            } else {
                AppError::from(error)
            }
        })?;
    drop(reservation);

    if let Err(error) = export_to_passphrase_database(&source, &destination, passphrase) {
        let _ = std::fs::remove_file(&destination);
        return Err(error);
    }
    if let Err(error) = open_portable_backup(&destination, passphrase, max_schema_version) {
        let _ = std::fs::remove_file(&destination);
        return Err(error);
    }
    Ok(destination)
}

fn export_to_passphrase_database(
    source: &Connection,
    destination: &Path,
    passphrase: &str,
) -> AppResult<()> {
    let schema_version: i32 = source.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    let attach = format!(
        "ATTACH DATABASE {} AS portable KEY {};",
        sql_string_literal(&destination.to_string_lossy()),
        sql_string_literal(passphrase)
    );
    source.execute_batch(&attach)?;
    let export_result = (|| -> rusqlite::Result<()> {
        source.execute_batch("SELECT sqlcipher_export('portable');")?;
        source.pragma_update(
            Some(rusqlite::DatabaseName::Attached("portable")),
            "user_version",
            schema_version,
        )
    })();
    let detach_result = source.execute_batch("DETACH DATABASE portable;");
    if let Err(error) = export_result {
        let _ = source.execute_batch("DETACH DATABASE portable;");
        return Err(AppError::from(error));
    }
    detach_result?;

    let file = std::fs::OpenOptions::new().write(true).open(destination)?;
    file.sync_all()?;
    Ok(())
}

fn validate_passphrase(passphrase: &str) -> AppResult<()> {
    if passphrase.trim().chars().count() < MIN_EXPORT_PASSWORD_LEN {
        return Err(AppError::validation(format!(
            "Use a backup passphrase with at least {MIN_EXPORT_PASSWORD_LEN} characters"
        )));
    }
    if passphrase.len() > 1024 {
        return Err(AppError::validation(
            "The backup passphrase must be 1024 characters or fewer",
        ));
    }
    Ok(())
}

/// Opens and validates a portable passphrase-protected SQLCipher database.
pub fn open_portable_backup(
    backup_path: &Path,
    passphrase: &str,
    max_schema_version: i32,
) -> AppResult<Connection> {
    validate_passphrase(passphrase)?;
    let source = Connection::open(backup_path)?;
    set_passphrase_key(&source, passphrase)?;
    validate_database(&source, max_schema_version)?;
    Ok(source)
}

pub fn restore_from_connection(source: &Connection, conn: &mut Connection) -> AppResult<()> {
    let backup = Backup::new(source, conn)?;
    backup.run_to_completion(128, Duration::from_millis(10), None)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sql_string_literals_escape_embedded_quotes() {
        assert_eq!(sql_string_literal("school's backup"), "'school''s backup'");
    }

    #[test]
    fn portable_passphrases_require_at_least_twelve_characters() {
        assert!(validate_passphrase("short").is_err());
        assert!(validate_passphrase("a longer passphrase").is_ok());
        assert!(validate_passphrase(&"a".repeat(1025)).is_err());
    }
}
