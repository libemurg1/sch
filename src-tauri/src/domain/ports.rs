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
    fn create(&self, class: &Class) -> AppResult<()>;
}

pub trait TermRepository {
    fn list(&self) -> AppResult<Vec<Term>>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Term>>;
    fn get_active(&self) -> AppResult<Option<Term>>;
    fn create(&self, term: &Term) -> AppResult<()>;
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

pub trait PaymentRepository {
    fn create(&self, payment: &Payment, allocations: &[PaymentAllocation]) -> AppResult<()>;
    fn find_by_id(&self, id: &str) -> AppResult<Option<Payment>>;
    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Payment>>;
    fn list_allocations(&self, payment_id: &str) -> AppResult<Vec<PaymentAllocation>>;
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
