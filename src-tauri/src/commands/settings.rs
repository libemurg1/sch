use crate::application::settings_service::SettingsService;
use crate::domain::auth::{Permission, SchoolSettings, UpdateSchoolSettingsInput};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

/// Public so the login screen can brand itself before authentication.
#[tauri::command]
pub fn get_school_settings(state: State<'_, AppState>) -> AppResult<SchoolSettings> {
    SettingsService::get(&state.db)
}

#[tauri::command]
pub fn update_school_settings(
    state: State<'_, AppState>,
    input: UpdateSchoolSettingsInput,
) -> AppResult<SchoolSettings> {
    let _user = state.require_write_permission(Permission::ManageSettings)?;
    SettingsService::update(&state.db, &input)
}

#[tauri::command]
pub fn set_logo(
    state: State<'_, AppState>,
    logo_data: Option<String>,
) -> AppResult<SchoolSettings> {
    let _user = state.require_write_permission(Permission::ManageSettings)?;
    SettingsService::set_logo(&state.db, logo_data)
}
