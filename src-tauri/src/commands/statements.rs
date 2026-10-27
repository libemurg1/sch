use crate::application::billing_service::BillingService;
use crate::application::credit_service::CreditService;
use crate::application::payment_service::PaymentService;
use crate::application::student_service::StudentService;
use crate::domain::dto::{AccountLedgerEntry, InvoiceDetail, StudentStatement};
use crate::domain::entities::{Invoice, Payment, Term};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::TermRepository;
use crate::AppState;
use rusqlite::Connection;
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

    let total_billed_cents: i64 = invoices
        .iter()
        .filter(|invoice| invoice.status != "CANCELLED")
        .map(|invoice| invoice.total_cents)
        .sum();
    let total_paid_cents: i64 = invoices
        .iter()
        .filter(|invoice| invoice.status != "CANCELLED")
        .map(|invoice| invoice.paid_cents)
        .sum();
    let account = CreditService::summary(&state.db, &student_id)?;
    let ledger = state
        .db
        .with_conn(|conn| build_ledger(conn, term_id.as_deref(), &invoices, &payments))?;

    let term: Option<Term> = match &term_id {
        Some(term_id) => state.db.with_conn(|conn| {
            let repository =
                crate::infrastructure::sqlite::term_repository::SqliteTermRepository::new(conn);
            repository.find_by_id(term_id)
        })?,
        None => None,
    };

    Ok(StudentStatement {
        student,
        term,
        invoices,
        payments,
        ledger,
        total_billed_cents,
        total_paid_cents,
        credit_cents: account.credit_cents,
        balance_cents: if term_id.is_some() {
            total_billed_cents - total_paid_cents
        } else {
            total_billed_cents - total_paid_cents - account.credit_cents
        },
    })
}

fn build_ledger(
    conn: &Connection,
    term_id: Option<&str>,
    invoices: &[Invoice],
    payments: &[Payment],
) -> AppResult<Vec<AccountLedgerEntry>> {
    let mut rows: Vec<(
        String,
        u8,
        String,
        String,
        i64,
        i64,
        Option<String>,
        Option<String>,
    )> = Vec::new();

    for invoice in invoices {
        rows.push((
            invoice.issue_date.clone(),
            1,
            invoice.invoice_number.clone(),
            format!("{} invoice", invoice.kind.to_lowercase()),
            invoice.total_cents,
            0,
            Some(invoice.id.clone()),
            None,
        ));
        if let Some(cancelled_at) = &invoice.cancelled_at {
            rows.push((
                cancelled_at.chars().take(10).collect(),
                3,
                invoice.invoice_number.clone(),
                format!(
                    "Invoice cancelled: {}",
                    invoice
                        .cancel_reason
                        .as_deref()
                        .unwrap_or("No reason supplied")
                ),
                0,
                invoice.total_cents,
                Some(invoice.id.clone()),
                None,
            ));
        }
    }

    for payment in payments {
        if term_id.is_none() {
            rows.push((
                payment.payment_date.clone(),
                2,
                payment.receipt_number.clone(),
                format!(
                    "Payment received ({})",
                    payment.payment_method.replace('_', " ")
                ),
                0,
                payment.amount_cents,
                None,
                Some(payment.id.clone()),
            ));
            if let Some(reversed_at) = &payment.reversed_at {
                rows.push((
                    reversed_at.chars().take(10).collect(),
                    4,
                    payment.receipt_number.clone(),
                    format!(
                        "Payment reversed: {}",
                        payment
                            .reversal_reason
                            .as_deref()
                            .unwrap_or("No reason supplied")
                    ),
                    payment.amount_cents,
                    0,
                    None,
                    Some(payment.id.clone()),
                ));
            }
            continue;
        }

        let allocations = {
            let mut statement = conn.prepare(
                "SELECT a.amount_cents, a.reversed_at, i.term_id, i.invoice_number \
                 FROM payment_allocations a \
                 JOIN invoices i ON i.id = a.invoice_id \
                 WHERE a.payment_id = ? ORDER BY a.id",
            )?;
            let mapped = statement.query_map([&payment.id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })?;
            let mut allocations = Vec::new();
            for allocation in mapped {
                allocations.push(allocation?);
            }
            allocations
        };

        for (amount_cents, allocation_reversed_at, allocation_term_id, invoice_number) in
            allocations
        {
            if Some(allocation_term_id.as_str()) != term_id {
                continue;
            }
            if payment.reversed_at.is_some() || allocation_reversed_at.is_none() {
                rows.push((
                    payment.payment_date.clone(),
                    2,
                    format!("{} / {}", payment.receipt_number, invoice_number),
                    format!(
                        "Payment allocated ({})",
                        payment.payment_method.replace('_', " ")
                    ),
                    0,
                    amount_cents,
                    None,
                    Some(payment.id.clone()),
                ));
            }
            if payment.reversed_at.is_some() {
                rows.push((
                    payment
                        .reversed_at
                        .as_deref()
                        .unwrap_or_default()
                        .chars()
                        .take(10)
                        .collect(),
                    4,
                    format!("{} / {}", payment.receipt_number, invoice_number),
                    format!(
                        "Payment reversed: {}",
                        payment
                            .reversal_reason
                            .as_deref()
                            .unwrap_or("No reason supplied")
                    ),
                    amount_cents,
                    0,
                    None,
                    Some(payment.id.clone()),
                ));
            } else if let Some(reversed_at) = allocation_reversed_at {
                rows.push((
                    reversed_at.chars().take(10).collect(),
                    4,
                    format!("{} / {}", payment.receipt_number, invoice_number),
                    "Payment allocation reversed".to_string(),
                    amount_cents,
                    0,
                    None,
                    Some(payment.id.clone()),
                ));
            }
        }
    }

    rows.sort_by(|left, right| (&left.0, left.1, &left.2).cmp(&(&right.0, right.1, &right.2)));

    let mut balance_cents = 0;
    Ok(rows
        .into_iter()
        .enumerate()
        .map(
            |(
                index,
                (
                    date,
                    sequence,
                    reference,
                    description,
                    debit_cents,
                    credit_cents,
                    invoice_id,
                    payment_id,
                ),
            )| {
                balance_cents += debit_cents - credit_cents;
                AccountLedgerEntry {
                    id: format!("{}-{}-{}-{}", date, sequence, reference, index),
                    date,
                    entry_type: if debit_cents > 0 {
                        "DEBIT".to_string()
                    } else {
                        "CREDIT".to_string()
                    },
                    reference,
                    description,
                    debit_cents,
                    credit_cents,
                    balance_cents,
                    invoice_id,
                    payment_id,
                }
            },
        )
        .collect())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::payment_service::PaymentService;
    use crate::domain::dto::RecordPaymentInput;
    use crate::domain::ports::{InvoiceRepository, PaymentRepository};
    use crate::infrastructure::db::Db;
    use crate::infrastructure::sqlite::invoice_repository::SqliteInvoiceRepository;
    use crate::infrastructure::sqlite::payment_repository::SqlitePaymentRepository;
    use std::fs;

    #[test]
    fn ledger_balances_account_credits_and_term_allocations() {
        let dir = std::env::temp_dir().join(format!(
            "sfs-ledger-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO students (id, student_number, first_name, last_name, dob, admission_date) \
                 VALUES ('s1', 'STU-1', 'Test', 'Student', '2015-01-01', '2026-01-01')",
                [],
            )?;
            conn.execute(
                "INSERT INTO terms (id, year, term_number, start_date, end_date) \
                 VALUES ('t1', 2026, 1, '2026-01-01', '2026-04-01')",
                [],
            )?;
            conn.execute(
                "INSERT INTO invoices (id, invoice_number, kind, student_id, term_id, issue_date, \
                 due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents) \
                 VALUES ('i1', 'INV-1', 'SUPPLEMENTARY', 's1', 't1', '2026-01-01', \
                 '2026-02-01', 'UNPAID', 4000, 0, 4000, 0)",
                [],
            )?;
            Ok(())
        })
        .unwrap();
        PaymentService::record(
            &db,
            RecordPaymentInput {
                student_id: "s1".to_string(),
                amount_cents: 6_000,
                payment_date: "2026-01-02".to_string(),
                payment_method: "CASH".to_string(),
                reference_number: None,
                created_by: None,
            },
        )
        .unwrap();

        let (invoices, payments) = db
            .with_conn(|conn| {
                Ok((
                    SqliteInvoiceRepository::new(conn).list_by_student("s1")?,
                    SqlitePaymentRepository::new(conn).list_by_student("s1")?,
                ))
            })
            .unwrap();
        let all_terms = db
            .with_conn(|conn| build_ledger(conn, None, &invoices, &payments))
            .unwrap();
        assert_eq!(all_terms.len(), 2);
        assert_eq!(all_terms[0].debit_cents, 4_000);
        assert_eq!(all_terms[1].credit_cents, 6_000);
        assert_eq!(all_terms[1].balance_cents, -2_000);

        let term = db
            .with_conn(|conn| build_ledger(conn, Some("t1"), &invoices, &payments))
            .unwrap();
        assert_eq!(term.len(), 2);
        assert_eq!(term[1].credit_cents, 4_000);
        assert_eq!(term[1].balance_cents, 0);

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }
}
