use crate::application::credit_service::CreditService;
use crate::application::{current_year, now_iso};
use crate::domain::dto::RecordPaymentInput;
use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::PaymentRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::number_generator::NumberGenerator;
use crate::infrastructure::sqlite::payment_repository::SqlitePaymentRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

const PAYMENT_METHODS: [&str; 4] = ["CASH", "MOBILE_MONEY", "BANK_TRANSFER", "CHEQUE"];

/// Use cases for recording money received from a student.
pub struct PaymentService;

impl PaymentService {
    /// Records money received into the student's account.
    ///
    /// A payment does not have to name an invoice: it can be recorded before
    /// any billing exists, or cover several future terms. Whatever is not
    /// applied to an open invoice immediately stays on the account as credit
    /// and is applied automatically as later invoices are raised.
    ///
    /// The payment and every allocation it causes are written in a single
    /// transaction, so a failure never leaves money without a receipt trail.
    pub fn record(db: &Db, input: RecordPaymentInput) -> AppResult<Payment> {
        if input.amount_cents <= 0 {
            return Err(AppError::validation("Payment amount must be positive"));
        }
        if input.payment_date.trim().is_empty() {
            return Err(AppError::validation("Payment date is required"));
        }
        if !PAYMENT_METHODS.contains(&input.payment_method.as_str()) {
            return Err(AppError::validation(format!(
                "Payment method must be one of {:?}",
                PAYMENT_METHODS
            )));
        }

        db.execute(|conn| {
            let payments = SqlitePaymentRepository::new(conn);
            let numbers = NumberGenerator::new(conn);

            if let Some(reference) = &input.reference_number {
                if input.payment_method != "CASH" && !reference.trim().is_empty() {
                    let existing: Option<i64> = conn
                        .query_row(
                            "SELECT COUNT(*) FROM payments WHERE payment_method = ? AND reference_number = ?",
                            rusqlite::params![input.payment_method, reference.trim()],
                            |row| row.get(0),
                        )
                        .ok();
                    if existing.unwrap_or(0) > 0 {
                        return Err(AppError::conflict(format!(
                            "Reference {} has already been used for {}",
                            reference.trim(),
                            input.payment_method
                        )));
                    }
                }
            }

            let payment = Payment {
                id: Uuid::new_v4().to_string(),
                receipt_number: numbers.next_formatted("REC", current_year(), 4)?,
                student_id: input.student_id.clone(),
                amount_cents: input.amount_cents,
                payment_date: input.payment_date.trim().to_string(),
                payment_method: input.payment_method.clone(),
                reference_number: input
                    .reference_number
                    .as_ref()
                    .map(|reference| reference.trim().to_string())
                    .filter(|reference| !reference.is_empty()),
                created_by: input.created_by,
                created_at: now_iso(),
            };

            payments.create(&payment, &[])?;
            CreditService::apply_conn(conn, &payment.student_id)?;

            Ok(payment)
        })
    }

    pub fn list(db: &Db) -> AppResult<Vec<Payment>> {
        db.with_conn(|conn| SqlitePaymentRepository::new(conn).list())
    }

    pub fn list_by_student(db: &Db, student_id: &str) -> AppResult<Vec<Payment>> {
        db.with_conn(|conn| SqlitePaymentRepository::new(conn).list_by_student(student_id))
    }

    pub fn find(db: &Db, id: &str) -> AppResult<Option<Payment>> {
        db.with_conn(|conn| SqlitePaymentRepository::new(conn).find_by_id(id))
    }

    pub fn list_allocations(
        db: &Db,
        payment_id: &str,
    ) -> AppResult<Vec<PaymentAllocation>> {
        db.with_conn(|conn| SqlitePaymentRepository::new(conn).list_allocations(payment_id))
    }
}
