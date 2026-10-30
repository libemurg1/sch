use crate::application::audit_service::AuditService;
use crate::application::credit_service::CreditService;
use crate::application::discount_calculator::{self, ChargeLine};
use crate::application::{current_year, now_iso};
use crate::domain::dto::{AssignStudentFeeInput, GenerateBatchInput, GenerateSupplementaryInput};
use crate::domain::entities::{Invoice, InvoiceItem, StudentFee};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::money::Money;
use crate::domain::ports::{
    DiscountRepository, EnrollmentRepository, FeeStructureRepository, InvoiceRepository,
    StudentFeeRepository,
};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::discount_repository::SqliteDiscountRepository;
use crate::infrastructure::sqlite::enrollment_repository::SqliteEnrollmentRepository;
use crate::infrastructure::sqlite::fee_structure_repository::SqliteFeeStructureRepository;
use crate::infrastructure::sqlite::invoice_repository::SqliteInvoiceRepository;
use crate::infrastructure::sqlite::number_generator::NumberGenerator;
use crate::infrastructure::sqlite::student_fee_repository::SqliteStudentFeeRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use serde_json::json;
use uuid::Uuid;

/// Use cases for raising invoices.
pub struct BillingService;

impl BillingService {
    /// Creates one `TERM` invoice per actively enrolled student of a class,
    /// snapshotting the class fee structure and applying the student's
    /// discounts.
    ///
    /// The whole batch is a single transaction: either every student is
    /// invoiced or none is.
    pub fn generate_batch(
        db: &Db,
        input: GenerateBatchInput,
        actor_id: &str,
    ) -> AppResult<Vec<Invoice>> {
        if input.issue_date.trim().is_empty() || input.due_date.trim().is_empty() {
            return Err(AppError::validation("Issue date and due date are required"));
        }
        if input.due_date < input.issue_date {
            return Err(AppError::validation(
                "Due date cannot precede the issue date",
            ));
        }

        db.execute(|conn| {
            let fee_structures = SqliteFeeStructureRepository::new(conn);
            let enrollments = SqliteEnrollmentRepository::new(conn);
            let invoices = SqliteInvoiceRepository::new(conn);
            let discount_repository = SqliteDiscountRepository::new(conn);
            let student_fees = SqliteStudentFeeRepository::new(conn);
            let numbers = NumberGenerator::new(conn);

            let structure = fee_structures
                .find_by_class_term(&input.class_id, &input.term_id)?
                .ok_or_else(|| {
                    AppError::new(
                        "NO_FEE_STRUCTURE",
                        "No fee structure has been set for this class and term",
                    )
                })?;

            let fee_items = fee_structures.list_items(&structure.id)?;
            let active_discounts = discount_repository.list_active()?;

            let enrolled = enrollments.list_by_class_term(&input.class_id, &input.term_id)?;
            let mut created = Vec::new();

            for enrollment in enrolled {
                if enrollment.status != "ACTIVE" {
                    continue;
                }

                // One live TERM invoice per student per term.
                if invoices
                    .find_live_by_student_term(&enrollment.student_id, &input.term_id)?
                    .is_some()
                {
                    continue;
                }

                let pending_student_fees =
                    student_fees.list_pending(&enrollment.student_id, &input.term_id)?;
                let mut charges: Vec<ChargeLine> = fee_items
                    .iter()
                    .map(|item| ChargeLine {
                        fee_item_id: Some(item.id.clone()),
                        student_fee_id: None,
                        description: item.description.clone(),
                        amount_cents: item.amount_cents,
                        discountable: item.discountable,
                    })
                    .collect();
                charges.extend(pending_student_fees.iter().map(|fee| ChargeLine {
                    fee_item_id: None,
                    student_fee_id: Some(fee.id.clone()),
                    description: fee.description.clone(),
                    amount_cents: fee.amount_cents,
                    discountable: fee.discountable,
                }));

                let granted =
                    discount_repository.list_for_student(&enrollment.student_id, &input.term_id)?;

                let discount_lines =
                    discount_calculator::calculate(&charges, &granted, &active_discounts);

                let subtotal = charges
                    .iter()
                    .map(|charge| Money::from_cents(charge.amount_cents))
                    .fold(Money::zero(), Money::add);
                let discount = discount_lines
                    .iter()
                    .map(|line| Money::from_cents(-line.amount_cents))
                    .fold(Money::zero(), Money::add);
                let subtotal_cents = subtotal.cents();
                let discount_cents = discount.cents();
                let total_cents = subtotal.sub(discount).max(Money::zero()).cents();

                let mut items: Vec<InvoiceItem> = charges
                    .iter()
                    .enumerate()
                    .map(|(index, charge)| InvoiceItem {
                        id: Uuid::new_v4().to_string(),
                        invoice_id: String::new(),
                        kind: if charge.student_fee_id.is_some() {
                            "STUDENT_FEE".to_string()
                        } else {
                            "FEE".to_string()
                        },
                        description: charge.description.clone(),
                        amount_cents: charge.amount_cents,
                        discountable: charge.discountable,
                        discount_id: None,
                        sort_order: index as i32,
                    })
                    .collect();

                for (sort_order, line) in (items.len() as i32..).zip(&discount_lines) {
                    items.push(InvoiceItem {
                        id: Uuid::new_v4().to_string(),
                        invoice_id: String::new(),
                        kind: "DISCOUNT".to_string(),
                        description: line.name.clone(),
                        amount_cents: line.amount_cents,
                        discountable: false,
                        discount_id: Some(line.discount_id.clone()),
                        sort_order,
                    });
                }

                let invoice = Invoice {
                    id: Uuid::new_v4().to_string(),
                    invoice_number: numbers.next_formatted("INV", current_year(), 3)?,
                    kind: "TERM".to_string(),
                    student_id: enrollment.student_id.clone(),
                    term_id: input.term_id.clone(),
                    fee_structure_id: Some(structure.id.clone()),
                    issue_date: input.issue_date.clone(),
                    due_date: input.due_date.clone(),
                    status: "UNPAID".to_string(),
                    subtotal_cents,
                    discount_cents,
                    total_cents,
                    paid_cents: 0,
                    cancelled_at: None,
                    cancelled_by: None,
                    cancel_reason: None,
                };

                for item in items.iter_mut() {
                    item.invoice_id = invoice.id.clone();
                }

                invoices.create(&invoice)?;
                invoices.create_items(&invoice.id, &items)?;
                for fee in &pending_student_fees {
                    student_fees.mark_billed(&fee.id, &invoice.id)?;
                }
                AuditService::record(
                    conn,
                    Some(actor_id),
                    "invoice",
                    &invoice.id,
                    "created",
                    json!({
                        "invoiceNumber": invoice.invoice_number,
                        "studentId": invoice.student_id,
                        "termId": invoice.term_id,
                        "totalCents": invoice.total_cents,
                        "kind": invoice.kind,
                    }),
                    &now_iso(),
                )?;

                // Credit already paid into the account clears this new invoice.
                CreditService::apply_conn(conn, &invoice.student_id)?;

                created.push(invoice);
            }

            Ok(created)
        })
    }

    /// Raises a `SUPPLEMENTARY` invoice for the ad-hoc student fees that are
    /// still pending for a student in a term.
    ///
    /// Supplementary invoices carry no fee structure and only percentage
    /// discounts are re-applied to them - fixed discounts are not repeated.
    pub fn create_supplementary(
        db: &Db,
        input: GenerateSupplementaryInput,
        actor_id: &str,
    ) -> AppResult<Option<Invoice>> {
        db.execute(|conn| {
            let invoices = SqliteInvoiceRepository::new(conn);
            let student_fees = SqliteStudentFeeRepository::new(conn);
            let discount_repository = SqliteDiscountRepository::new(conn);
            let numbers = NumberGenerator::new(conn);

            let pending = student_fees.list_pending(&input.student_id, &input.term_id)?;
            if pending.is_empty() {
                return Ok(None);
            }

            let charges: Vec<ChargeLine> = pending
                .iter()
                .map(|fee| ChargeLine {
                    fee_item_id: None,
                    student_fee_id: Some(fee.id.clone()),
                    description: fee.description.clone(),
                    amount_cents: fee.amount_cents,
                    discountable: fee.discountable,
                })
                .collect();

            let granted =
                discount_repository.list_for_student(&input.student_id, &input.term_id)?;
            let active_discounts = discount_repository.list_active()?;
            let percentage_only: Vec<_> = granted
                .iter()
                .filter(|granted| {
                    active_discounts
                        .iter()
                        .find(|definition| definition.id == granted.discount_id)
                        .map(|definition| definition.kind == "PERCENT")
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            let discount_lines =
                discount_calculator::calculate(&charges, &percentage_only, &active_discounts);

            let subtotal = charges
                .iter()
                .map(|charge| Money::from_cents(charge.amount_cents))
                .fold(Money::zero(), Money::add);
            let discount = discount_lines
                .iter()
                .map(|line| Money::from_cents(-line.amount_cents))
                .fold(Money::zero(), Money::add);
            let subtotal_cents = subtotal.cents();
            let discount_cents = discount.cents();
            let total_cents = subtotal.sub(discount).max(Money::zero()).cents();

            let invoice = Invoice {
                id: Uuid::new_v4().to_string(),
                invoice_number: numbers.next_formatted("INV", current_year(), 3)?,
                kind: "SUPPLEMENTARY".to_string(),
                student_id: input.student_id.clone(),
                term_id: input.term_id.clone(),
                fee_structure_id: None,
                issue_date: input.issue_date.clone(),
                due_date: input.due_date.clone(),
                status: "UNPAID".to_string(),
                subtotal_cents,
                discount_cents,
                total_cents,
                paid_cents: 0,
                cancelled_at: None,
                cancelled_by: None,
                cancel_reason: None,
            };

            let mut items: Vec<InvoiceItem> = charges
                .iter()
                .enumerate()
                .map(|(index, charge)| InvoiceItem {
                    id: Uuid::new_v4().to_string(),
                    invoice_id: invoice.id.clone(),
                    kind: "STUDENT_FEE".to_string(),
                    description: charge.description.clone(),
                    amount_cents: charge.amount_cents,
                    discountable: charge.discountable,
                    discount_id: None,
                    sort_order: index as i32,
                })
                .collect();

            for (sort_order, line) in (items.len() as i32..).zip(&discount_lines) {
                items.push(InvoiceItem {
                    id: Uuid::new_v4().to_string(),
                    invoice_id: invoice.id.clone(),
                    kind: "DISCOUNT".to_string(),
                    description: line.name.clone(),
                    amount_cents: line.amount_cents,
                    discountable: false,
                    discount_id: Some(line.discount_id.clone()),
                    sort_order,
                });
            }

            invoices.create(&invoice)?;
            invoices.create_items(&invoice.id, &items)?;
            AuditService::record(
                conn,
                Some(actor_id),
                "invoice",
                &invoice.id,
                "created",
                json!({
                    "invoiceNumber": invoice.invoice_number,
                    "studentId": invoice.student_id,
                    "termId": invoice.term_id,
                    "totalCents": invoice.total_cents,
                    "kind": invoice.kind,
                }),
                &now_iso(),
            )?;

            for fee in &pending {
                student_fees.mark_billed(&fee.id, &invoice.id)?;
            }

            // Credit already paid into the account clears this new invoice.
            CreditService::apply_conn(conn, &invoice.student_id)?;

            Ok(Some(invoice))
        })
    }

    /// Records an ad-hoc charge against a student for a term.
    ///
    /// The charge stays `PENDING` until it is billed on a supplementary
    /// invoice, which is how late fees and other one-off charges reach a
    /// student without re-running the whole class batch.
    pub fn assign_student_fee(
        db: &Db,
        input: AssignStudentFeeInput,
        actor_id: &str,
    ) -> AppResult<StudentFee> {
        if input.description.trim().is_empty() {
            return Err(AppError::validation("Description is required"));
        }
        if input.amount_cents <= 0 {
            return Err(AppError::validation("Amount must be greater than zero"));
        }

        db.execute(|conn| {
            let student_fees = SqliteStudentFeeRepository::new(conn);

            let fee = StudentFee {
                id: Uuid::new_v4().to_string(),
                student_id: input.student_id.clone(),
                term_id: input.term_id.clone(),
                description: input.description.trim().to_string(),
                amount_cents: input.amount_cents,
                discountable: input.discountable,
                status: "PENDING".to_string(),
                invoice_id: None,
            };

            student_fees.create(&fee)?;
            AuditService::record(
                conn,
                Some(actor_id),
                "student_fee",
                &fee.id,
                "created",
                json!({
                    "studentId": fee.student_id,
                    "termId": fee.term_id,
                    "description": fee.description,
                    "amountCents": fee.amount_cents,
                }),
                &now_iso(),
            )?;
            Ok(fee)
        })
    }

    pub fn cancel_invoice(
        db: &Db,
        invoice_id: &str,
        actor_id: &str,
        reason: &str,
    ) -> AppResult<Invoice> {
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(AppError::validation(
                "A reason is required to cancel an invoice",
            ));
        }

        db.execute(|conn| {
            let invoices = SqliteInvoiceRepository::new(conn);
            let student_fees = SqliteStudentFeeRepository::new(conn);
            let at = now_iso();
            let invoice = invoices.cancel(invoice_id, actor_id, reason, &at)?;
            student_fees.reset_pending_for_invoice(invoice_id)?;
            AuditService::record(
                conn,
                Some(actor_id),
                "invoice",
                invoice_id,
                "cancelled",
                json!({
                    "invoiceNumber": invoice.invoice_number,
                    "studentId": invoice.student_id,
                    "totalCents": invoice.total_cents,
                    "reason": reason,
                }),
                &at,
            )?;
            Ok(invoice)
        })
    }

    pub fn list(db: &Db) -> AppResult<Vec<Invoice>> {
        db.with_conn(|conn| SqliteInvoiceRepository::new(conn).list())
    }

    pub fn find(db: &Db, id: &str) -> AppResult<Option<Invoice>> {
        db.with_conn(|conn| SqliteInvoiceRepository::new(conn).find_by_id(id))
    }

    pub fn list_items(db: &Db, invoice_id: &str) -> AppResult<Vec<InvoiceItem>> {
        db.with_conn(|conn| SqliteInvoiceRepository::new(conn).list_items(invoice_id))
    }

    pub fn list_by_student(db: &Db, student_id: &str) -> AppResult<Vec<Invoice>> {
        db.with_conn(|conn| SqliteInvoiceRepository::new(conn).list_by_student(student_id))
    }

    pub fn list_by_class_term(db: &Db, class_id: &str, term_id: &str) -> AppResult<Vec<Invoice>> {
        db.with_conn(|conn| {
            SqliteInvoiceRepository::new(conn).list_by_class_term(class_id, term_id)
        })
    }

    /// Convenience helper used by the setup command to report on the database.
    pub fn count(db: &Db) -> AppResult<i64> {
        db.with_conn(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::dto::GenerateBatchInput;
    use crate::infrastructure::db::Db;
    use std::fs;

    #[test]
    fn class_invoice_includes_only_that_students_pending_fees() {
        let dir = std::env::temp_dir().join(format!(
            "sfs-invoice-student-fee-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO users (id, username, password_hash, display_name) \
                 VALUES ('u1', 'admin', 'test-hash', 'Admin')",
                [],
            )?;
            conn.execute(
                "INSERT INTO students (id, student_number, first_name, last_name, dob, admission_date) \
                 VALUES ('s1', 'STU-1', 'Ada', 'One', '2015-01-01', '2026-01-01'), \
                        ('s2', 'STU-2', 'Grace', 'Two', '2015-01-01', '2026-01-01')",
                [],
            )?;
            conn.execute(
                "INSERT INTO terms (id, year, term_number, start_date, end_date) \
                 VALUES ('t1', 2026, 1, '2026-01-01', '2026-04-01')",
                [],
            )?;
            conn.execute(
                "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
                 VALUES ('c1', 'Grade 2 Blue', 2, 'Blue', 2026, 30)",
                [],
            )?;
            conn.execute(
                "INSERT INTO enrollments (id, student_id, class_id, term_id, status) \
                 VALUES ('e1', 's1', 'c1', 't1', 'ACTIVE'), \
                        ('e2', 's2', 'c1', 't1', 'ACTIVE')",
                [],
            )?;
            conn.execute(
                "INSERT INTO fee_structures (id, class_id, term_id, total_cents) \
                 VALUES ('fs1', 'c1', 't1', 1000)",
                [],
            )?;
            conn.execute(
                "INSERT INTO fee_items (id, fee_structure_id, description, amount_cents) \
                 VALUES ('fi1', 'fs1', 'Tuition', 1000)",
                [],
            )?;
            conn.execute(
                "INSERT INTO student_fees (id, student_id, term_id, description, amount_cents) \
                 VALUES ('f1', 's1', 't1', 'Trip fee', 500)",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let invoices = BillingService::generate_batch(
            &db,
            GenerateBatchInput {
                class_id: "c1".to_string(),
                term_id: "t1".to_string(),
                issue_date: "2026-01-01".to_string(),
                due_date: "2026-02-01".to_string(),
            },
            "u1",
        )
        .unwrap();

        assert_eq!(invoices.len(), 2);
        let invoice_one = invoices
            .iter()
            .find(|invoice| invoice.student_id == "s1")
            .unwrap();
        let invoice_two = invoices
            .iter()
            .find(|invoice| invoice.student_id == "s2")
            .unwrap();
        assert_eq!(invoice_one.subtotal_cents, 1500);
        assert_eq!(invoice_two.subtotal_cents, 1000);

        let (fee_status, fee_invoice_id, student_fee_item_count): (String, String, i64) = db
            .with_conn(|conn| {
                Ok((
                    conn.query_row(
                        "SELECT status FROM student_fees WHERE id = 'f1'",
                        [],
                        |row| row.get(0),
                    )?,
                    conn.query_row(
                        "SELECT invoice_id FROM student_fees WHERE id = 'f1'",
                        [],
                        |row| row.get(0),
                    )?,
                    conn.query_row(
                        "SELECT COUNT(*) FROM invoice_items \
                         WHERE invoice_id = ? AND kind = 'STUDENT_FEE'",
                        [&invoice_one.id],
                        |row| row.get(0),
                    )?,
                ))
            })
            .unwrap();
        assert_eq!(fee_status, "BILLED");
        assert_eq!(fee_invoice_id, invoice_one.id);
        assert_eq!(student_fee_item_count, 1);

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cancelling_an_unpaid_supplementary_invoice_releases_its_fees() {
        let dir = std::env::temp_dir().join(format!(
            "sfs-invoice-cancel-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_millis()
        ));
        let db = Db::open(&dir).unwrap();
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO users (id, username, password_hash, display_name) \
                 VALUES ('u1', 'admin', 'test-hash', 'Admin')",
                [],
            )?;
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
                 '2026-02-01', 'UNPAID', 1000, 0, 1000, 0)",
                [],
            )?;
            conn.execute(
                "INSERT INTO student_fees (id, student_id, term_id, description, amount_cents, \
                 status, invoice_id) VALUES ('f1', 's1', 't1', 'Late fee', 1000, 'BILLED', 'i1')",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let invoice = BillingService::cancel_invoice(&db, "i1", "u1", "Incorrect charge").unwrap();
        assert_eq!(invoice.status, "CANCELLED");
        let (fee_status, audit_count): (String, i64) = db
            .with_conn(|conn| {
                Ok((
                    conn.query_row(
                        "SELECT status FROM student_fees WHERE id = 'f1'",
                        [],
                        |row| row.get(0),
                    )?,
                    conn.query_row(
                        "SELECT COUNT(*) FROM audit_log \
                         WHERE entity = 'invoice' AND action = 'cancelled'",
                        [],
                        |row| row.get(0),
                    )?,
                ))
            })
            .unwrap();
        assert_eq!(fee_status, "PENDING");
        assert_eq!(audit_count, 1);

        drop(db);
        let _ = fs::remove_dir_all(dir);
    }
}
