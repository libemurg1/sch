use crate::application::{current_year, now_iso};
use crate::domain::dto::{PaymentAllocationInput, RecordPaymentInput};
use crate::domain::entities::{Payment, PaymentAllocation};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::{InvoiceRepository, PaymentRepository};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::invoice_repository::SqliteInvoiceRepository;
use crate::infrastructure::sqlite::number_generator::NumberGenerator;
use crate::infrastructure::sqlite::payment_repository::SqlitePaymentRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

const PAYMENT_METHODS: [&str; 4] = ["CASH", "MOBILE_MONEY", "BANK_TRANSFER", "CHEQUE"];

/// Use cases for recording payments against invoices.
pub struct PaymentService;

impl PaymentService {
    /// Records a payment and allocates it across one or more invoices.
    ///
    /// The payment, its allocations and the resulting invoice balances are
    /// written in a single transaction, so a failure never leaves a payment
    /// without its allocations.
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
        if input.allocations.is_empty() {
            return Err(AppError::validation(
                "A payment must be allocated to at least one invoice",
            ));
        }

        let allocated: i64 = input
            .allocations
            .iter()
            .map(|allocation| allocation.amount_cents)
            .sum();
        if allocated != input.amount_cents {
            return Err(AppError::validation(format!(
                "Allocations total {} but the payment is for {}",
                allocated, input.amount_cents
            )));
        }
        if input.allocations.iter().any(|a| a.amount_cents <= 0) {
            return Err(AppError::validation(
                "Allocation amounts must be positive",
            ));
        }

        db.execute(|conn| {
            let payments = SqlitePaymentRepository::new(conn);
            let invoices = SqliteInvoiceRepository::new(conn);
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

            // Validate every invoice before writing anything.
            for allocation in &input.allocations {
                let invoice = invoices
                    .find_by_id(&allocation.invoice_id)?
                    .ok_or_else(|| AppError::not_found("Invoice", &allocation.invoice_id))?;

                if invoice.student_id != input.student_id {
                    return Err(AppError::validation(format!(
                        "Invoice {} does not belong to this student",
                        invoice.invoice_number
                    )));
                }
                if invoice.status == "CANCELLED" {
                    return Err(AppError::conflict(format!(
                        "Invoice {} is cancelled",
                        invoice.invoice_number
                    )));
                }

                let outstanding = invoice.total_cents - invoice.paid_cents;
                if allocation.amount_cents > outstanding {
                    return Err(AppError::conflict(format!(
                        "Allocation of {} exceeds the outstanding balance of {} on invoice {}",
                        allocation.amount_cents, outstanding, invoice.invoice_number
                    )));
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

            let allocations: Vec<PaymentAllocation> = input
                .allocations
                .iter()
                .map(|allocation: &PaymentAllocationInput| PaymentAllocation {
                    id: Uuid::new_v4().to_string(),
                    payment_id: payment.id.clone(),
                    invoice_id: allocation.invoice_id.clone(),
                    amount_cents: allocation.amount_cents,
                })
                .collect();

            payments.create(&payment, &allocations)?;

            for allocation in &allocations {
                invoices.apply_payment(&allocation.invoice_id, allocation.amount_cents)?;
            }

            Ok(payment)
        })
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
