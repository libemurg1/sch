use crate::application::promotion_service::PromotionService;
use crate::domain::auth::Permission;
use crate::domain::dto::{ApplyPromotionInput, PromotionPreview, PromotionResult};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn copy_classes(state: State<'_, AppState>, from_year: i32, to_year: i32) -> AppResult<i32> {
    let _user = state.require_write_permission(Permission::ManageEnrollment)?;
    PromotionService::copy_classes(&state.db, from_year, to_year)
}

#[tauri::command]
pub fn preview_promotion(
    state: State<'_, AppState>,
    from_year: i32,
    to_year: i32,
) -> AppResult<PromotionPreview> {
    let _user = state.require_auth()?;
    PromotionService::preview(&state.db, from_year, to_year)
}

#[tauri::command]
pub fn apply_promotion(
    state: State<'_, AppState>,
    input: ApplyPromotionInput,
) -> AppResult<PromotionResult> {
    let user = state.require_write_permission(Permission::ManageEnrollment)?;
    PromotionService::apply(&state.db, input, Some(&user.id))
}
