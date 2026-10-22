use crate::domain::errors::AppError;
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

pub fn backup_db(app_handle: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| AppError {
        code: "PATH_ERROR".to_string(),
        message: e.to_string(),
    })?;
    fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("school.db");
    let backup_path = app_dir.join(format!(
        "school_backup_{}.db",
        chrono::Utc::now().format("%Y%m%d_%H%M%S")
    ));
    fs::copy(&db_path, &backup_path)?;
    Ok(backup_path)
}

pub fn restore_db(app_handle: &tauri::AppHandle, backup_path: &PathBuf) -> Result<(), AppError> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| AppError {
        code: "PATH_ERROR".to_string(),
        message: e.to_string(),
    })?;
    fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("school.db");
    fs::copy(backup_path, &db_path)?;
    Ok(())
}
