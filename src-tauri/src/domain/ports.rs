pub trait StudentRepository {
    fn create(&self, student: &crate::domain::entities::Student) -> crate::domain::errors::AppResult<()>;
    fn find_by_id(&self, id: &str) -> crate::domain::errors::AppResult<Option<crate::domain::entities::Student>>;
    fn list(&self) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Student>>;
}

pub trait ClassRepository {
    fn list(&self) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Class>>;
    fn create(&self, class: &crate::domain::entities::Class) -> crate::domain::errors::AppResult<()>;
}

pub trait TermRepository {
    fn list(&self) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Term>>;
    fn get_active(&self) -> crate::domain::errors::AppResult<Option<crate::domain::entities::Term>>;
    fn create(&self, term: &crate::domain::entities::Term) -> crate::domain::errors::AppResult<()>;
}

pub trait EnrollmentRepository {
    fn create(&self, enrollment: &crate::domain::entities::Enrollment) -> crate::domain::errors::AppResult<()>;
    fn find_by_student_term(&self, student_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Option<crate::domain::entities::Enrollment>>;
    fn count_by_class_term(&self, class_id: &str, term_id: &str) -> crate::domain::errors::AppResult<i32>;
    fn list_by_class_term(&self, class_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Enrollment>>;
}

pub trait FeeStructureRepository {
    fn find_by_class_term(&self, class_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Option<crate::domain::entities::FeeStructure>>;
    fn save(&self, structure: &crate::domain::entities::FeeStructure, items: &[crate::domain::entities::FeeItem]) -> crate::domain::errors::AppResult<()>;
}

pub trait InvoiceRepository {
    fn create(&self, invoice: &crate::domain::entities::Invoice) -> crate::domain::errors::AppResult<()>;
    fn create_items(&self, invoice_id: &str, items: &[crate::domain::entities::InvoiceItem]) -> crate::domain::errors::AppResult<()>;
    fn find_live_by_student_term(&self, student_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Option<crate::domain::entities::Invoice>>;
    fn list(&self) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Invoice>>;
    fn find_by_id(&self, id: &str) -> crate::domain::errors::AppResult<Option<crate::domain::entities::Invoice>>;
}

pub trait PaymentRepository {
    fn create(&self, payment: &crate::domain::entities::Payment, allocations: &[crate::domain::entities::PaymentAllocation]) -> crate::domain::errors::AppResult<()>;
}

pub trait StudentFeeRepository {
    fn create(&self, fee: &crate::domain::entities::StudentFee) -> crate::domain::errors::AppResult<()>;
    fn list_pending(&self, student_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::StudentFee>>;
    fn mark_billed(&self, fee_id: &str, invoice_id: &str) -> crate::domain::errors::AppResult<()>;
}

pub trait DiscountRepository {
    fn list_active(&self) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::Discount>>;
    fn list_for_student(&self, student_id: &str, term_id: &str) -> crate::domain::errors::AppResult<Vec<crate::domain::entities::StudentDiscount>>;
}
