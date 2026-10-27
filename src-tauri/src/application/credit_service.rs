use crate::domain::dto::StudentAccount;
use crate::domain::entities::PaymentAllocation;
use crate::domain::errors::AppResult;
use crate::domain::ports::{CreditSource, InvoiceRepository, PaymentRepository};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::invoice_repository::SqliteInvoiceRepository;
use crate::infrastructure::sqlite::payment_repository::SqlitePaymentRepository;
use rusqlite::Connection;
use uuid::Uuid;

/// Applies money already received onto invoices.
///
/// A payment is money paid into a student's account, not money tied to one
/// invoice. Anything that has not been applied yet is the student's credit. It
/// is applied to the oldest unpaid invoice first, as soon as that invoice
/// exists - which is what makes paying for a whole year before the later terms
/// are billed work.
pub struct CreditService;

impl CreditService {
    /// Applies any available credit to the student's open invoices and returns
    /// what is left over.
    pub fn apply_conn(conn: &Connection, student_id: &str) -> AppResult<i64> {
        let payments = SqlitePaymentRepository::new(conn);
        let invoices = SqliteInvoiceRepository::new(conn);

        let mut sources: Vec<CreditSource> = payments
            .credit_sources(student_id)?
            .into_iter()
            .filter(|source| source.remaining_cents > 0)
            .collect();
        if sources.is_empty() {
            return Ok(0);
        }

        let mut open: Vec<_> = invoices
            .list_by_student(student_id)?
            .into_iter()
            .filter(|invoice| {
                invoice.status != "CANCELLED" && invoice.paid_cents < invoice.total_cents
            })
            .collect();
        open.sort_by(|left, right| {
            (&left.due_date, &left.issue_date, &left.invoice_number).cmp(&(
                &right.due_date,
                &right.issue_date,
                &right.invoice_number,
            ))
        });

        for invoice in open {
            let mut outstanding = invoice.total_cents - invoice.paid_cents;

            for source in sources.iter_mut() {
                if outstanding == 0 {
                    break;
                }
                if source.remaining_cents == 0 {
                    continue;
                }

                let amount = outstanding.min(source.remaining_cents);
                payments.add_allocation(&PaymentAllocation {
                    id: Uuid::new_v4().to_string(),
                    payment_id: source.payment_id.clone(),
                    invoice_id: invoice.id.clone(),
                    amount_cents: amount,
                    reversed_at: None,
                    reversal_reason: None,
                })?;
                invoices.apply_payment(&invoice.id, amount)?;

                source.remaining_cents -= amount;
                outstanding -= amount;
            }
        }

        Ok(sources.iter().map(|source| source.remaining_cents).sum())
    }

    /// Billed-but-unpaid, unapplied credit and the net balance for a student.
    pub fn summary(db: &Db, student_id: &str) -> AppResult<StudentAccount> {
        db.with_conn(|conn| {
            let invoices = SqliteInvoiceRepository::new(conn);
            let payments = SqlitePaymentRepository::new(conn);

            let outstanding_cents: i64 = invoices
                .list_by_student(student_id)?
                .into_iter()
                .filter(|invoice| invoice.status != "CANCELLED")
                .map(|invoice| invoice.total_cents - invoice.paid_cents)
                .sum();

            let credit_cents: i64 = payments
                .credit_sources(student_id)?
                .into_iter()
                .map(|source| source.remaining_cents)
                .sum();

            Ok(StudentAccount {
                student_id: student_id.to_string(),
                outstanding_cents,
                credit_cents,
                balance_cents: outstanding_cents - credit_cents,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::payment_service::PaymentService;
    use crate::domain::dto::RecordPaymentInput;
    use crate::domain::entities::Payment;
    use crate::infrastructure::sqlite::invoice_repository::SqliteInvoiceRepository;
    use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (Db, PathBuf) {
        let dir = std::env::temp_dir().join(format!("sfs-credit-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let db = Db::open(&dir).unwrap();
        (db, dir)
    }

    fn seed_student_and_term(db: &Db) {
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
            Ok(())
        })
        .unwrap();
    }

    fn add_invoice(db: &Db, id: &str, number: &str, due: &str, total_cents: i64) {
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO invoices (id, invoice_number, kind, student_id, term_id, issue_date, \
                 due_date, status, subtotal_cents, discount_cents, total_cents, paid_cents) \
                 VALUES (?, ?, 'SUPPLEMENTARY', 's1', 't1', '2026-01-01', ?, 'UNPAID', ?, 0, ?, 0)",
                rusqlite::params![id, number, due, total_cents, total_cents],
            )?;
            Ok(())
        })
        .unwrap();
    }

    fn pay(db: &Db, amount_cents: i64, date: &str) -> Payment {
        PaymentService::record(
            db,
            RecordPaymentInput {
                student_id: "s1".to_string(),
                amount_cents,
                payment_date: date.to_string(),
                payment_method: "CASH".to_string(),
                reference_number: None,
                created_by: None,
            },
        )
        .unwrap()
    }

    fn invoice(db: &Db, id: &str) -> crate::domain::entities::Invoice {
        db.with_conn(|conn| SqliteInvoiceRepository::new(conn).find_by_id(id))
            .unwrap()
            .unwrap()
    }

    #[test]
    fn a_prepayment_is_held_as_credit_until_it_is_billed() {
        let (db, dir) = temp_db("prepay");
        seed_student_and_term(&db);

        pay(&db, 100_000, "2026-01-05");

        let account = CreditService::summary(&db, "s1").unwrap();
        assert_eq!(account.outstanding_cents, 0);
        assert_eq!(account.credit_cents, 100_000);
        assert_eq!(account.balance_cents, -100_000);

        // The first term is billed: the credit clears it immediately.
        add_invoice(&db, "i1", "INV-1", "2026-03-01", 40_000);
        db.execute(|conn| CreditService::apply_conn(conn, "s1"))
            .unwrap();

        let first = invoice(&db, "i1");
        assert_eq!(first.paid_cents, 40_000);
        assert_eq!(first.status, "PAID");

        let account = CreditService::summary(&db, "s1").unwrap();
        assert_eq!(account.outstanding_cents, 0);
        assert_eq!(account.credit_cents, 60_000);

        // A later term is billed and also covered.
        add_invoice(&db, "i2", "INV-2", "2026-06-01", 50_000);
        db.execute(|conn| CreditService::apply_conn(conn, "s1"))
            .unwrap();

        let second = invoice(&db, "i2");
        assert_eq!(second.paid_cents, 50_000);
        assert_eq!(second.status, "PAID");

        let account = CreditService::summary(&db, "s1").unwrap();
        assert_eq!(account.credit_cents, 10_000);
        assert_eq!(account.outstanding_cents, 0);

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn a_payment_clears_the_oldest_invoice_first() {
        let (db, dir) = temp_db("oldest");
        seed_student_and_term(&db);

        add_invoice(&db, "late", "INV-LATE", "2026-09-01", 30_000);
        add_invoice(&db, "early", "INV-EARLY", "2026-03-01", 20_000);

        pay(&db, 40_000, "2026-02-01");

        let early = invoice(&db, "early");
        assert_eq!(early.status, "PAID");
        assert_eq!(early.paid_cents, 20_000);

        let late = invoice(&db, "late");
        assert_eq!(late.status, "PARTIAL");
        assert_eq!(late.paid_cents, 20_000);

        let account = CreditService::summary(&db, "s1").unwrap();
        assert_eq!(account.outstanding_cents, 10_000);
        assert_eq!(account.credit_cents, 0);
        assert_eq!(account.balance_cents, 10_000);

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn reversing_a_payment_restores_invoice_balance_and_audit_reason() {
        let (db, dir) = temp_db("reverse");
        seed_student_and_term(&db);
        add_invoice(&db, "i1", "INV-1", "2026-03-01", 40_000);
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO users (id, username, password_hash, display_name) \
                 VALUES ('u1', 'admin', 'test-hash', 'Admin')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let payment = pay(&db, 25_000, "2026-02-01");
        PaymentService::reverse(&db, &payment.id, "u1", "Duplicate receipt").unwrap();

        let invoice = invoice(&db, "i1");
        assert_eq!(invoice.paid_cents, 0);
        assert_eq!(invoice.status, "UNPAID");

        let account = CreditService::summary(&db, "s1").unwrap();
        assert_eq!(account.outstanding_cents, 40_000);
        assert_eq!(account.credit_cents, 0);
        assert_eq!(account.balance_cents, 40_000);

        let (reversed_at, reason): (Option<String>, Option<String>) = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT reversed_at, reversal_reason FROM payments WHERE id = ?",
                    [&payment.id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?)
            })
            .unwrap();
        assert!(reversed_at.is_some());
        assert_eq!(reason.as_deref(), Some("Duplicate receipt"));

        let audit_reason: String = db
            .with_conn(|conn| {
                Ok(conn.query_row(
                    "SELECT json_extract(detail_json, '$.reason') FROM audit_log \
                     WHERE entity = 'payment' AND action = 'reversed'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .unwrap();
        assert_eq!(audit_reason, "Duplicate receipt");

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }
}
