use crate::domain::auth::{SchoolSettings, User};
use crate::domain::entities::{
    Class, Discount, Enrollment, FeeItem, FeeStructure, Invoice, InvoiceItem, Payment,
    PaymentAllocation, Student, StudentDiscount, StudentFee, Term,
};
use crate::domain::errors::AppResult;

/// Ports describing the persistence capabilities the application layer needs.
/// Implementations live in `infrastructure::sqlite` and borrow a `&Connection`
/// so that they can participate in a caller's transaction.
pub trait StudentRepository {
    fn create(&self, student: &Student) -> AppResult<()>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Student>>;
    fn list(&self) -> AppResult<Vec<Student>>;
    fn list_by_status(&self, status: &str) -> AppResult<Vec<Student>>;
    fn update_status(&self, id: &str, status: &str) -> AppResult<()>;
}

pub trait ClassRepository {
    fn list(&self) -> AppResult<Vec<Class>>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Class>>;
    fn find_by_name(&self, name: &str) -> AppResult<Option<Class>>;
    fn create(&self, class: &Class) -> AppResult<()>;
    fn update(&self, class: &Class) -> AppResult<()>;
    fn delete(&self, id: &str) -> AppResult<()>;
    /// Number of records (enrollments, fee structures, invoices) using this class.
    fn reference_count(&self, id: &str) -> AppResult<i64>;
}

pub trait TermRepository {
    fn list(&self) -> AppResult<Vec<Term>>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Term>>;
    fn find_by_year_term(&self, year: i32, term_number: i32) -> AppResult<Option<Term>>;
    fn get_active(&self) -> AppResult<Option<Term>>;
    fn create(&self, term: &Term) -> AppResult<()>;
    fn update(&self, term: &Term) -> AppResult<()>;
    fn delete(&self, id: &str) -> AppResult<()>;
    /// Clears the active flag everywhere except `id`.
    fn deactivate_others(&self, id: &str) -> AppResult<()>;
    /// Number of records (enrollments, fees, invoices, discounts) using this term.
    fn reference_count(&self, id: &str) -> AppResult<i64>;
}

pub trait EnrollmentRepository {
    fn create(&self, enrollment: &Enrollment) -> AppResult<()>;
    fn find_by_student_term(
        &self,
        student_id: &str,
        term_id: &str,
    ) -> AppResult<Option<Enrollment>>;
    fn count_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<i32>;
    fn list_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<Vec<Enrollment>>;
    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Enrollment>>;
}

pub trait FeeStructureRepository {
    fn find_by_class_term(
        &self,
        class_id: &str,
        term_id: &str,
    ) -> AppResult<Option<FeeStructure>>;
    fn list_items(&self, fee_structure_id: &str) -> AppResult<Vec<FeeItem>>;
    /// Creates or replaces a fee structure together with its items.
    fn save(&self, structure: &FeeStructure, items: &[FeeItem]) -> AppResult<()>;
}

pub trait InvoiceRepository {
    fn create(&self, invoice: &Invoice) -> AppResult<()>;
    fn create_items(&self, invoice_id: &str, items: &[InvoiceItem]) -> AppResult<()>;
    fn list_items(&self, invoice_id: &str) -> AppResult<Vec<InvoiceItem>>;
    fn find_live_by_student_term(
        &self,
        student_id: &str,
        term_id: &str,
    ) -> AppResult<Option<Invoice>>;
    fn list(&self) -> AppResult<Vec<Invoice>>;
    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Invoice>>;
    fn list_by_class_term(
        &self,
        class_id: &str,
        term_id: &str,
    ) -> AppResult<Vec<Invoice>>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Invoice>>;
    /// Adds `amount_cents` to the invoice's paid amount and recomputes status.
    fn apply_payment(&self, invoice_id: &str, amount_cents: i64) -> AppResult<Invoice>;
}

/// Money that has been received but not yet applied to an invoice.
#[derive(Debug, Clone)]
pub struct CreditSource {
    pub payment_id: String,
    pub remaining_cents: i64,
}

pub trait PaymentRepository {
    fn create(&self, payment: &Payment, allocations: &[PaymentAllocation]) -> AppResult<()>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Payment>>;
    fn list(&self) -> AppResult<Vec<Payment>>;
    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Payment>>;
    fn list_allocations(&self, payment_id: &str) -> AppResult<Vec<PaymentAllocation>>;
    /// Adds a single allocation to an existing payment.
    fn add_allocation(&self, allocation: &PaymentAllocation) -> AppResult<()>;
    /// Unallocated payments for a student, oldest first.
    fn credit_sources(&self, student_id: &str) -> AppResult<Vec<CreditSource>>;
}

pub trait StudentFeeRepository {
    fn create(&self, fee: &StudentFee) -> AppResult<()>;
    fn list_pending(&self, student_id: &str, term_id: &str) -> AppResult<Vec<StudentFee>>;
    fn mark_billed(&self, fee_id: &str, invoice_id: &str) -> AppResult<()>;
}

pub trait DiscountRepository {
    fn list_active(&self) -> AppResult<Vec<Discount>>;
    fn list_for_student(&self, student_id: &str, term_id: &str) -> AppResult<Vec<StudentDiscount>>;
    fn create(&self, discount: &Discount) -> AppResult<()>;
    fn assign(&self, student_discount: &StudentDiscount) -> AppResult<()>;
}

pub trait UserRepository {
    fn create(&self, user: &User, password_hash: &str) -> AppResult<()>;
    fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;
    fn find_by_id(&self, user_id: &str) -> AppResult<Option<User>>;
    fn password_hash(&self, user_id: &str) -> AppResult<Option<String>>;
    fn list(&self) -> AppResult<Vec<User>>;
    fn count(&self) -> AppResult<i64>;
    fn update_password(&self, user_id: &str, password_hash: &str) -> AppResult<()>;
    fn update_profile(&self, user_id: &str, username: &str, display_name: &str) -> AppResult<()>;
    fn set_active(&self, user_id: &str, is_active: bool) -> AppResult<()>;
    fn record_login(&self, user_id: &str, at: &str) -> AppResult<()>;
    fn delete(&self, user_id: &str) -> AppResult<()>;
}

pub trait SettingsRepository {
    fn get(&self) -> AppResult<SchoolSettings>;
    fn save(&self, settings: &SchoolSettings) -> AppResult<()>;
    fn save_logo(&self, logo_data: Option<&str>) -> AppResult<()>;
}
