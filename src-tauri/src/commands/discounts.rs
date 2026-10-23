use crate::AppState;
use crate::application::discount_service::DiscountService;
use crate::domain::dto::{AssignDiscountInput, CreateDiscountInput};
use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::errors::AppResult;
use tauri::State;

#[tauri::command]
pub fn list_discounts(state: State<'_, AppState>) -> AppResult<Vec<Discount>> {
    let _user = state.require_auth()?;
    DiscountService::list_active(&state.db)
}

#[tauri::command]
pub fn create_discount(state: State<'_, AppState>, input: CreateDiscountInput) -> AppResult<Discount> {
    let _user = state.require_auth()?;
    DiscountService::create(&state.db, input)
}

#[tauri::command]
pub fn list_student_discounts(
    state: State<'_, AppState>,
    student_id: String,
    term_id: String,
) -> AppResult<Vec<StudentDiscount>> {
    let _user = state.require_auth()?;
    DiscountService::list_for_student(&state.db, &student_id, &term_id)
}

#[tauri::command]
pub fn assign_discount(
    state: State<'_, AppState>,
    input: AssignDiscountInput,
) -> AppResult<StudentDiscount> {
    let _user = state.require_auth()?;
    DiscountService::assign(&state.db, input)
}
