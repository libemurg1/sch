use crate::AppState;
use crate::application::billing_service::BillingService;
use crate::application::credit_service::CreditService;
use crate::application::enrollment_service::EnrollmentService;
use crate::application::payment_service::PaymentService;
use crate::application::student_service::StudentService;
use crate::domain::dto::{InvoiceDetail, StudentStatement};
use crate::domain::entities::Term;
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::TermRepository;
use tauri::State;

/// Builds the statement shown for a student: their invoices, payments and
/// the resulting balance.
#[tauri::command]
pub fn get_student_statement(
    state: State<'_, AppState>,
    student_id: String,
    term_id: Option<String>,
) -> AppResult<StudentStatement> {
    let _user = state.require_auth()?;
    let student = StudentService::find(&state.db, &student_id)?
        .ok_or_else(|| AppError::not_found("Student", &student_id))?;

    let invoices = match &term_id {
        Some(term_id) => BillingService::list_by_student(&state.db, &student_id)?
            .into_iter()
            .filter(|invoice| &invoice.term_id == term_id)
            .collect(),
        None => BillingService::list_by_student(&state.db, &student_id)?,
    };

    let payments = PaymentService::list_by_student(&state.db, &student_id)?;

    let total_billed_cents: i64 = invoices.iter().map(|invoice| invoice.total_cents).sum();
    let total_paid_cents: i64 = invoices.iter().map(|invoice| invoice.paid_cents).sum();
    let account = CreditService::summary(&state.db, &student_id)?;

    let term: Option<Term> = match &term_id {
        Some(term_id) => state.db.with_conn(|conn| {
            let repository = crate::infrastructure::sqlite::term_repository::SqliteTermRepository::new(conn);
            repository.find_by_id(term_id)
        })?,
        None => EnrollmentService::get_active_term(&state.db)?,
    };

    Ok(StudentStatement {
        student,
        term,
        invoices,
        payments,
        total_billed_cents,
        total_paid_cents,
        credit_cents: account.credit_cents,
        balance_cents: total_billed_cents - total_paid_cents - account.credit_cents,
    })
}

/// Returns an invoice together with its line items, ready for printing.
#[tauri::command]
pub fn get_invoice_detail(
    state: State<'_, AppState>,
    invoice_id: String,
) -> AppResult<InvoiceDetail> {
    let _user = state.require_auth()?;
    let invoice = BillingService::find(&state.db, &invoice_id)?
        .ok_or_else(|| AppError::not_found("Invoice", &invoice_id))?;
    let items = BillingService::list_items(&state.db, &invoice_id)?;
    Ok(InvoiceDetail { invoice, items })
}
