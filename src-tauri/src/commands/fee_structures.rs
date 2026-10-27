use crate::application::fee_structure_service::FeeStructureService;
use crate::domain::dto::SaveFeeStructureInput;
use crate::domain::entities::{FeeItem, FeeStructure};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_fee_structure(
    state: State<'_, AppState>,
    class_id: String,
    term_id: String,
) -> AppResult<Option<(FeeStructure, Vec<FeeItem>)>> {
    let _user = state.require_auth()?;
    FeeStructureService::get(&state.db, &class_id, &term_id)
}

#[tauri::command]
pub fn save_fee_structure(
    state: State<'_, AppState>,
    input: SaveFeeStructureInput,
) -> AppResult<FeeStructure> {
    let _user = state.require_write_auth()?;
    FeeStructureService::save(&state.db, input)
}
