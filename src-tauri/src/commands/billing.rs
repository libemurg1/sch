use crate::application::billing_service::BillingService;
use crate::domain::auth::Permission;
use crate::domain::dto::{AssignStudentFeeInput, GenerateBatchInput, GenerateSupplementaryInput};
use crate::domain::entities::{Invoice, InvoiceItem, StudentFee};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn generate_invoice_batch(
    state: State<'_, AppState>,
    input: GenerateBatchInput,
) -> AppResult<Vec<Invoice>> {
    let user = state.require_write_permission(Permission::ManageBilling)?;
    BillingService::generate_batch(&state.db, input, &user.id)
}

#[tauri::command]
pub fn generate_supplementary_invoice(
    state: State<'_, AppState>,
    input: GenerateSupplementaryInput,
) -> AppResult<Option<Invoice>> {
    let user = state.require_write_permission(Permission::ManageBilling)?;
    BillingService::create_supplementary(&state.db, input, &user.id)
}

/// Records an ad-hoc charge (a late fee, for example) against a student.
#[tauri::command]
pub fn assign_student_fee(
    state: State<'_, AppState>,
    input: AssignStudentFeeInput,
) -> AppResult<StudentFee> {
    let user = state.require_write_permission(Permission::ManageBilling)?;
    BillingService::assign_student_fee(&state.db, input, &user.id)
}

#[tauri::command]
pub fn cancel_invoice(
    state: State<'_, AppState>,
    invoice_id: String,
    reason: String,
) -> AppResult<Invoice> {
    let user = state.require_write_permission(Permission::CancelInvoices)?;
    BillingService::cancel_invoice(&state.db, &invoice_id, &user.id, &reason)
}

#[tauri::command]
pub fn list_invoices(state: State<'_, AppState>) -> AppResult<Vec<Invoice>> {
    let _user = state.require_auth()?;
    BillingService::list(&state.db)
}

#[tauri::command]
pub fn list_student_invoices(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Invoice>> {
    let _user = state.require_auth()?;
    BillingService::list_by_student(&state.db, &student_id)
}

#[tauri::command]
pub fn list_class_invoices(
    state: State<'_, AppState>,
    class_id: String,
    term_id: String,
) -> AppResult<Vec<Invoice>> {
    let _user = state.require_auth()?;
    BillingService::list_by_class_term(&state.db, &class_id, &term_id)
}

#[tauri::command]
pub fn get_invoice(state: State<'_, AppState>, id: String) -> AppResult<Option<Invoice>> {
    let _user = state.require_auth()?;
    BillingService::find(&state.db, &id)
}

#[tauri::command]
pub fn get_invoice_items(
    state: State<'_, AppState>,
    invoice_id: String,
) -> AppResult<Vec<InvoiceItem>> {
    let _user = state.require_auth()?;
    BillingService::list_items(&state.db, &invoice_id)
}
