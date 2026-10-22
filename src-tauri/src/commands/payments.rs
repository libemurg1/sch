use crate::AppState;
use crate::application::payment_service::PaymentService;
use crate::domain::dto::RecordPaymentInput;
use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::AppResult;
use tauri::State;

#[tauri::command]
pub fn record_payment(state: State<'_, AppState>, input: RecordPaymentInput) -> AppResult<Payment> {
    PaymentService::record(&state.db, input)
}

#[tauri::command]
pub fn list_student_payments(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Payment>> {
    PaymentService::list_by_student(&state.db, &student_id)
}

#[tauri::command]
pub fn get_payment(state: State<'_, AppState>, id: String) -> AppResult<Option<Payment>> {
    PaymentService::find(&state.db, &id)
}

#[tauri::command]
pub fn get_payment_allocations(
    state: State<'_, AppState>,
    payment_id: String,
) -> AppResult<Vec<PaymentAllocation>> {
    PaymentService::list_allocations(&state.db, &payment_id)
}
