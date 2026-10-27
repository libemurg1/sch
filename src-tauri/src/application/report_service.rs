use crate::application::today;
use crate::domain::dto::{FinanceReport, PaymentMethodSummary, TermBalanceSummary};
use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::db::Db;
use chrono::NaiveDate;

pub struct ReportService;

impl ReportService {
    pub fn finance(db: &Db, start_date: &str, end_date: &str) -> AppResult<FinanceReport> {
        let start = NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
            .map_err(|_| AppError::validation("Start date must use YYYY-MM-DD format"))?;
        let end = NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
            .map_err(|_| AppError::validation("End date must use YYYY-MM-DD format"))?;
        if start > end {
            return Err(AppError::validation("Start date cannot be after end date"));
        }

        db.with_conn(|conn| {
            let mut statement = conn.prepare(
                "SELECT payment_method, COUNT(*), COALESCE(SUM(amount_cents), 0) \
                 FROM payments \
                 WHERE reversed_at IS NULL AND payment_date BETWEEN ? AND ? \
                 GROUP BY payment_method ORDER BY payment_method",
            )?;
            let mapped = statement.query_map([start_date, end_date], |row| {
                Ok(PaymentMethodSummary {
                    payment_method: row.get(0)?,
                    payment_count: row.get(1)?,
                    amount_cents: row.get(2)?,
                })
            })?;
            let mut by_method = Vec::new();
            for row in mapped {
                by_method.push(row?);
            }
            let payment_count = by_method.iter().map(|row| row.payment_count).sum();
            let payments_received_cents = by_method.iter().map(|row| row.amount_cents).sum();

            let report_date = today();
            let mut statement = conn.prepare(
                "SELECT t.id, t.year, t.term_number, COUNT(i.id), \
                 COALESCE(SUM(i.total_cents), 0), COALESCE(SUM(i.paid_cents), 0), \
                 COALESCE(SUM(CASE WHEN i.due_date < ? \
                   THEN i.total_cents - i.paid_cents ELSE 0 END), 0) \
                 FROM terms t LEFT JOIN invoices i \
                   ON i.term_id = t.id AND i.status <> 'CANCELLED' \
                 GROUP BY t.id, t.year, t.term_number \
                 HAVING COUNT(i.id) > 0 ORDER BY t.year DESC, t.term_number DESC",
            )?;
            let mapped = statement.query_map([&report_date], |row| {
                let year: i32 = row.get(1)?;
                let term_number: i32 = row.get(2)?;
                let billed_cents: i64 = row.get(4)?;
                let paid_cents: i64 = row.get(5)?;
                Ok(TermBalanceSummary {
                    term_id: row.get(0)?,
                    label: format!("Term {}, {}", term_number, year),
                    invoice_count: row.get(3)?,
                    billed_cents,
                    paid_cents,
                    outstanding_cents: billed_cents - paid_cents,
                    overdue_cents: row.get(6)?,
                })
            })?;
            let mut by_term = Vec::new();
            for row in mapped {
                by_term.push(row?);
            }

            Ok(FinanceReport {
                start_date: start_date.to_string(),
                end_date: end_date.to_string(),
                payment_count,
                payments_received_cents,
                outstanding_cents: by_term.iter().map(|row| row.outstanding_cents).sum(),
                overdue_cents: by_term.iter().map(|row| row.overdue_cents).sum(),
                by_method,
                by_term,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::payment_service::PaymentService;
    use crate::domain::dto::RecordPaymentInput;
    use crate::infrastructure::db::Db;
    use std::fs;

    #[test]
    fn finance_report_summarizes_collection_and_overdue_balances() {
        let dir = std::env::temp_dir().join(format!(
            "sfs-report-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        let db = Db::open(&dir).unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let yesterday = (chrono::Utc::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
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
                 ?, 'UNPAID', 10000, 0, 10000, 0)",
                [&yesterday],
            )?;
            Ok(())
        })
        .unwrap();
        PaymentService::record(
            &db,
            RecordPaymentInput {
                student_id: "s1".to_string(),
                amount_cents: 2_500,
                payment_date: today.clone(),
                payment_method: "CASH".to_string(),
                reference_number: None,
                created_by: None,
            },
        )
        .unwrap();

        let report = ReportService::finance(&db, &today, &today).unwrap();
        assert_eq!(report.payment_count, 1);
        assert_eq!(report.payments_received_cents, 2_500);
        assert_eq!(report.by_method.len(), 1);
        assert_eq!(report.by_method[0].payment_method, "CASH");
        assert_eq!(report.outstanding_cents, 7_500);
        assert_eq!(report.overdue_cents, 7_500);
        assert_eq!(report.by_term[0].label, "Term 1, 2026");

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }
}
