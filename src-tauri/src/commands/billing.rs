use crate::AppState;
use crate::application::billing_service::BillingService;
use crate::domain::dto::{AssignStudentFeeInput, GenerateBatchInput, GenerateSupplementaryInput};
use crate::domain::entities::{Invoice, InvoiceItem, StudentFee};
use crate::domain::errors::AppResult;
use tauri::State;

#[tauri::command]
pub fn generate_invoice_batch(
    state: State<'_, AppState>,
    input: GenerateBatchInput,
) -> AppResult<Vec<Invoice>> {
    BillingService::generate_batch(&state.db, input)
}

#[tauri::command]
pub fn generate_supplementary_invoice(
    state: State<'_, AppState>,
    input: GenerateSupplementaryInput,
) -> AppResult<Option<Invoice>> {
    BillingService::create_supplementary(&state.db, input)
}

/// Records an ad-hoc charge (a late fee, for example) against a student.
#[tauri::command]
pub fn assign_student_fee(
    state: State<'_, AppState>,
    input: AssignStudentFeeInput,
) -> AppResult<StudentFee> {
    BillingService::assign_student_fee(&state.db, input)
}

#[tauri::command]
pub fn list_invoices(state: State<'_, AppState>) -> AppResult<Vec<Invoice>> {
    BillingService::list(&state.db)
}

#[tauri::command]
pub fn list_student_invoices(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Invoice>> {
    BillingService::list_by_student(&state.db, &student_id)
}

#[tauri::command]
pub fn list_class_invoices(
    state: State<'_, AppState>,
    class_id: String,
    term_id: String,
) -> AppResult<Vec<Invoice>> {
    BillingService::list_by_class_term(&state.db, &class_id, &term_id)
}

#[tauri::command]
pub fn get_invoice(state: State<'_, AppState>, id: String) -> AppResult<Option<Invoice>> {
    BillingService::find(&state.db, &id)
}

#[tauri::command]
pub fn get_invoice_items(
    state: State<'_, AppState>,
    invoice_id: String,
) -> AppResult<Vec<InvoiceItem>> {
    BillingService::list_items(&state.db, &invoice_id)
}
