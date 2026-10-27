use crate::application::audit_service::AuditService;
use crate::application::report_service::ReportService;
use crate::domain::dto::{AuditEntry, FinanceReport};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_finance_report(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> AppResult<FinanceReport> {
    let _user = state.require_auth()?;
    ReportService::finance(&state.db, &start_date, &end_date)
}

#[tauri::command]
pub fn list_audit_entries(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> AppResult<Vec<AuditEntry>> {
    let _user = state.require_auth()?;
    state
        .db
        .with_conn(|conn| AuditService::list(conn, limit.unwrap_or(200).clamp(1, 500)))
}
