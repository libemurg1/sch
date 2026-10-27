use crate::application::credit_service::CreditService;
use crate::application::payment_service::PaymentService;
use crate::domain::dto::{RecordPaymentInput, StudentAccount};
use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn record_payment(
    state: State<'_, AppState>,
    mut input: RecordPaymentInput,
) -> AppResult<Payment> {
    let user = state.require_auth()?;
    // The audit trail is authoritative: it always reflects the signed-in user.
    input.created_by = Some(user.id);
    PaymentService::record(&state.db, input)
}

/// Every payment recorded, newest first.
#[tauri::command]
pub fn list_payments(state: State<'_, AppState>) -> AppResult<Vec<Payment>> {
    let _user = state.require_auth()?;
    PaymentService::list(&state.db)
}

#[tauri::command]
pub fn list_student_payments(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Payment>> {
    let _user = state.require_auth()?;
    PaymentService::list_by_student(&state.db, &student_id)
}

#[tauri::command]
pub fn get_payment(state: State<'_, AppState>, id: String) -> AppResult<Option<Payment>> {
    let _user = state.require_auth()?;
    PaymentService::find(&state.db, &id)
}

#[tauri::command]
pub fn get_payment_allocations(
    state: State<'_, AppState>,
    payment_id: String,
) -> AppResult<Vec<PaymentAllocation>> {
    let _user = state.require_auth()?;
    PaymentService::list_allocations(&state.db, &payment_id)
}

/// What a student owes, what is held as credit, and the net balance.
#[tauri::command]
pub fn get_student_account(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<StudentAccount> {
    let _user = state.require_auth()?;
    CreditService::summary(&state.db, &student_id)
}

#[tauri::command]
pub fn reverse_payment(
    state: State<'_, AppState>,
    payment_id: String,
    reason: String,
) -> AppResult<Payment> {
    let user = state.require_auth()?;
    PaymentService::reverse(&state.db, &payment_id, &user.id, &reason)
}
