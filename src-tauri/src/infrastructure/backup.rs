use crate::domain::errors::{AppError, AppResult};
use rusqlite::backup::Backup;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Creates a consistent copy of the database using SQLite's online backup
/// API.
///
/// A plain file copy is not safe here: the database runs in WAL mode, so any
/// pages still sitting in the write-ahead log would be missing from the copy.
pub fn backup_database(conn: &Connection, app_dir: &Path) -> AppResult<PathBuf> {
    std::fs::create_dir_all(app_dir)?;

    let backup_path = app_dir.join(format!(
        "school_backup_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));

    let mut destination = Connection::open(&backup_path)?;
    let backup = Backup::new(conn, &mut destination)?;
    backup.run_to_completion(128, Duration::from_millis(10), None)?;

    Ok(backup_path)
}

/// Lists the backups stored in `app_dir`, newest first.
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

/// Restores the database from a backup file into the live connection.
///
/// The backup is copied into the open database with SQLite's online backup
/// API, running in reverse. This keeps the write-ahead log consistent and
/// means the application does not have to be closed and reopened.
pub fn restore_database(conn: &mut Connection, backup_path: &Path) -> AppResult<()> {
    if !backup_path.exists() {
        return Err(AppError::new(
            "BACKUP_FAILED",
            format!("Backup not found: {}", backup_path.display()),
        ));
    }

    let source = Connection::open(backup_path)?;
    let backup = Backup::new(&source, conn)?;
    backup.run_to_completion(128, Duration::from_millis(10), None)?;

    Ok(())
}
