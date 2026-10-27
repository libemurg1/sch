use crate::domain::errors::AppResult;
use crate::license::LicenseStatus;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_license_status(state: State<'_, AppState>) -> AppResult<LicenseStatus> {
    state.licensing.status()
}

#[tauri::command]
pub fn activate_license(
    state: State<'_, AppState>,
    license_key: String,
) -> AppResult<LicenseStatus> {
    let _user = state.require_auth()?;
    state.licensing.activate(&license_key)
}
