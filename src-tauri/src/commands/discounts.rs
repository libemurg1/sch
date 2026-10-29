use crate::application::discount_service::DiscountService;
use crate::domain::auth::Permission;
use crate::domain::dto::{AssignDiscountInput, CreateDiscountInput};
use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_discounts(state: State<'_, AppState>) -> AppResult<Vec<Discount>> {
    let _user = state.require_auth()?;
    DiscountService::list_active(&state.db)
}

#[tauri::command]
pub fn create_discount(
    state: State<'_, AppState>,
    input: CreateDiscountInput,
) -> AppResult<Discount> {
    let user = state.require_write_permission(Permission::ManageDiscounts)?;
    DiscountService::create(&state.db, input, &user.id)
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
    let user = state.require_write_permission(Permission::ManageDiscounts)?;
    DiscountService::assign(&state.db, input, &user.id)
}

#[tauri::command]
pub fn list_all_student_discounts(state: State<'_, AppState>) -> AppResult<Vec<StudentDiscount>> {
    let _user = state.require_auth()?;
    DiscountService::list_all_grants(&state.db)
}

#[tauri::command]
pub fn revoke_discount(
    state: State<'_, AppState>,
    grant_id: String,
    reason: String,
) -> AppResult<StudentDiscount> {
    let user = state.require_write_permission(Permission::ManageDiscounts)?;
    DiscountService::revoke(&state.db, &grant_id, &user.id, &reason)
}
