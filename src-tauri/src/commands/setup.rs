use crate::application::billing_service::BillingService;
use crate::application::today;
use crate::domain::dto::AppInfo;
use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::backup;
use crate::AppState;
use std::path::PathBuf;
use tauri::State;

/// The directory the database and its backups live in.
fn app_dir(state: &AppState) -> AppResult<PathBuf> {
    state
        .db_path
        .parent()
        .map(|path| path.to_path_buf())
        .ok_or_else(|| AppError::new("PATH_ERROR", "Invalid database path"))
}

/// Reports on the running application: version, database location and how much
/// data is currently stored.
#[tauri::command]
pub fn get_app_info(state: State<'_, AppState>) -> AppResult<AppInfo> {
    let _user = state.require_auth()?;
    let student_count = state.db.with_conn(|conn| {
        Ok(conn.query_row("SELECT COUNT(*) FROM students", [], |row| row.get(0))?)
    })?;
    let invoice_count = BillingService::count(&state.db)?;

    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        database_path: state.db_path.to_string_lossy().into_owned(),
        today: today(),
        student_count,
        invoice_count,
    })
}

/// Creates a consistent snapshot of the database and returns its path.
#[tauri::command]
pub fn backup_database(state: State<'_, AppState>) -> AppResult<String> {
    let _user = state.require_auth()?;
    let path = state.db.backup(&app_dir(&state)?)?;
    Ok(path.to_string_lossy().into_owned())
}

/// Lists the available backups, newest first.
#[tauri::command]
pub fn list_backups(state: State<'_, AppState>) -> AppResult<Vec<String>> {
    let _user = state.require_auth()?;
    let backups = backup::list_backups(&app_dir(&state)?)?;
    Ok(backups
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect())
}

/// Restores the database from a backup, replacing the current data.
#[tauri::command]
pub fn restore_database(
    state: State<'_, AppState>,
    backup_path: String,
    passphrase: Option<String>,
) -> AppResult<()> {
    let _user = state.require_auth()?;
    state
        .db
        .restore_from_backup(&PathBuf::from(backup_path), passphrase.as_deref())
}

/// Copies an application-created backup into a user-selected existing folder.
#[tauri::command]
pub fn export_backup(
    state: State<'_, AppState>,
    backup_path: String,
    destination_directory: String,
    passphrase: String,
) -> AppResult<String> {
    let _user = state.require_auth()?;
    let source = PathBuf::from(backup_path);
    let known_backups = backup::list_backups(&app_dir(&state)?)?;
    let source_canonical = source.canonicalize().map_err(|error| {
        AppError::new(
            "BACKUP_FAILED",
            format!("Cannot access backup source: {}", error),
        )
    })?;
    let mut is_known_backup = false;
    for path in known_backups {
        if path.canonicalize()? == source_canonical {
            is_known_backup = true;
            break;
        }
    }
    if !is_known_backup {
        return Err(AppError::new(
            "BACKUP_FAILED",
            "Only backups created by this application can be exported here",
        ));
    }

    let exported =
        state
            .db
            .export_backup(&source, &PathBuf::from(destination_directory), &passphrase)?;
    Ok(exported.to_string_lossy().into_owned())
}
