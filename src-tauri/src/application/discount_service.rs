use crate::domain::dto::{AssignDiscountInput, CreateDiscountInput};
use crate::domain::entities::{Discount, StudentDiscount};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::DiscountRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::discount_repository::SqliteDiscountRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

/// Use cases for discount definitions and the grants made to students.
pub struct DiscountService;

impl DiscountService {
    pub fn list_active(db: &Db) -> AppResult<Vec<Discount>> {
        db.with_conn(|conn| SqliteDiscountRepository::new(conn).list_active())
    }

    /// Creates a discount definition.
    ///
    /// `PERCENT` values are basis points (`1000` = 10.00%) and may not exceed
    /// `10000` (100%); `FIXED` values are cents.
    pub fn create(db: &Db, input: CreateDiscountInput) -> AppResult<Discount> {
        if input.name.trim().is_empty() {
            return Err(AppError::validation("Discount name is required"));
        }
        if input.value <= 0 {
            return Err(AppError::validation("Discount value must be positive"));
        }
        match input.kind.as_str() {
            "PERCENT" => {
                if input.value > 10_000 {
                    return Err(AppError::validation(
                        "Percentage discounts are capped at 10000 basis points (100%)",
                    ));
                }
            }
            "FIXED" => {}
            other => {
                return Err(AppError::validation(format!(
                    "Discount kind must be PERCENT or FIXED, got '{}'",
                    other
                )))
            }
        }

        let discount = Discount {
            id: Uuid::new_v4().to_string(),
            name: input.name.trim().to_string(),
            kind: input.kind.clone(),
            value: input.value,
            is_active: true,
        };

        db.execute(|conn| SqliteDiscountRepository::new(conn).create(&discount))?;
        Ok(discount)
    }

    pub fn list_for_student(
        db: &Db,
        student_id: &str,
        term_id: &str,
    ) -> AppResult<Vec<StudentDiscount>> {
        db.with_conn(|conn| SqliteDiscountRepository::new(conn).list_for_student(student_id, term_id))
    }

    /// Grants a discount to a student, optionally limited to a term or to a
    /// specific fee item.
    pub fn assign(db: &Db, input: AssignDiscountInput) -> AppResult<StudentDiscount> {
        if input.student_id.trim().is_empty() || input.discount_id.trim().is_empty() {
            return Err(AppError::validation(
                "Student and discount are both required",
            ));
        }
        if input.target_fee_item_id.is_some() && input.target_student_fee_id.is_some() {
            return Err(AppError::validation(
                "A discount can target either a fee item or a student fee, not both",
            ));
        }

        let grant = StudentDiscount {
            id: Uuid::new_v4().to_string(),
            student_id: input.student_id.trim().to_string(),
            discount_id: input.discount_id.trim().to_string(),
            term_id: input.term_id.map(|term| term.trim().to_string()).filter(|term| !term.is_empty()),
            target_fee_item_id: input
                .target_fee_item_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            target_student_fee_id: input
                .target_student_fee_id
                .map(|id| id.trim().to_string())
                .filter(|id| !id.is_empty()),
            reason: input.reason,
        };

        db.execute(|conn| SqliteDiscountRepository::new(conn).assign(&grant))?;
        Ok(grant)
    }
}
