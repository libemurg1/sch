use crate::domain::entities::{Invoice, InvoiceItem, Payment, Student, Term};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct RegisterStudentInput {
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    #[serde(default)]
    pub gender: Option<String>,
    pub admission_date: String,
    #[serde(default)]
    pub guardian_name: Option<String>,
    #[serde(default)]
    pub guardian_phone: Option<String>,
    #[serde(default)]
    pub guardian_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateStudentStatusInput {
    pub student_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateStudentInput {
    pub student_id: String,
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    #[serde(default)]
    pub gender: Option<String>,
    pub admission_date: String,
    #[serde(default)]
    pub guardian_name: Option<String>,
    #[serde(default)]
    pub guardian_phone: Option<String>,
    #[serde(default)]
    pub guardian_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateClassInput {
    pub grade_level: i32,
    #[serde(default)]
    pub stream: String,
    pub academic_year: i32,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateClassInput {
    pub id: String,
    pub grade_level: i32,
    #[serde(default)]
    pub stream: String,
    pub academic_year: i32,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateTermInput {
    pub year: i32,
    pub term_number: i32,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateTermInput {
    pub id: String,
    pub year: i32,
    pub term_number: i32,
    pub start_date: String,
    pub end_date: String,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct EnrollStudentInput {
    pub student_id: String,
    pub class_id: String,
    pub term_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct FeeItemInput {
    pub description: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    #[serde(default = "default_true")]
    pub discountable: bool,
    #[serde(default)]
    pub sort_order: i32,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct SaveFeeStructureInput {
    pub class_id: String,
    pub term_id: String,
    pub items: Vec<FeeItemInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct GenerateBatchInput {
    pub class_id: String,
    pub term_id: String,
    pub issue_date: String,
    pub due_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct GenerateSupplementaryInput {
    pub student_id: String,
    pub term_id: String,
    pub issue_date: String,
    pub due_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AssignStudentFeeInput {
    pub student_id: String,
    pub term_id: String,
    pub description: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    #[serde(default = "default_true")]
    pub discountable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct RecordPaymentInput {
    pub student_id: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub payment_date: String,
    pub payment_method: String,
    #[serde(default)]
    pub reference_number: Option<String>,
    #[serde(default)]
    pub created_by: Option<String>,
}

/// A student's running account: what they owe, what is held as credit, and the
/// net position. A negative balance means the student is in credit.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct StudentAccount {
    pub student_id: String,
    #[ts(as = "f64")]
    pub outstanding_cents: i64,
    #[ts(as = "f64")]
    pub credit_cents: i64,
    #[ts(as = "f64")]
    pub balance_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PaymentMethodSummary {
    pub payment_method: String,
    pub payment_count: i64,
    #[ts(as = "f64")]
    pub amount_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct TermBalanceSummary {
    pub term_id: String,
    pub label: String,
    pub invoice_count: i64,
    #[ts(as = "f64")]
    pub billed_cents: i64,
    #[ts(as = "f64")]
    pub paid_cents: i64,
    #[ts(as = "f64")]
    pub outstanding_cents: i64,
    #[ts(as = "f64")]
    pub overdue_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct FinanceReport {
    pub start_date: String,
    pub end_date: String,
    pub payment_count: i64,
    #[ts(as = "f64")]
    pub payments_received_cents: i64,
    #[ts(as = "f64")]
    pub outstanding_cents: i64,
    #[ts(as = "f64")]
    pub overdue_cents: i64,
    pub by_method: Vec<PaymentMethodSummary>,
    pub by_term: Vec<TermBalanceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AuditEntry {
    pub id: i64,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub entity: String,
    pub entity_id: String,
    pub action: String,
    pub detail_json: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateDiscountInput {
    pub name: String,
    pub kind: String,
    #[ts(as = "f64")]
    pub value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AssignDiscountInput {
    pub student_id: String,
    pub discount_id: String,
    #[serde(default)]
    pub term_id: Option<String>,
    #[serde(default)]
    pub target_fee_item_id: Option<String>,
    #[serde(default)]
    pub target_student_fee_id: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct StudentStatement {
    pub student: Student,
    pub term: Option<Term>,
    pub invoices: Vec<Invoice>,
    pub payments: Vec<Payment>,
    pub ledger: Vec<AccountLedgerEntry>,
    #[ts(as = "f64")]
    pub total_billed_cents: i64,
    #[ts(as = "f64")]
    pub total_paid_cents: i64,
    #[ts(as = "f64")]
    pub credit_cents: i64,
    #[ts(as = "f64")]
    pub balance_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AccountLedgerEntry {
    pub id: String,
    pub date: String,
    pub entry_type: String,
    pub reference: String,
    pub description: String,
    #[ts(as = "f64")]
    pub debit_cents: i64,
    #[ts(as = "f64")]
    pub credit_cents: i64,
    #[ts(as = "f64")]
    pub balance_cents: i64,
    pub invoice_id: Option<String>,
    pub payment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct InvoiceDetail {
    pub invoice: Invoice,
    pub items: Vec<InvoiceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AppInfo {
    pub version: String,
    pub database_path: String,
    /// The server's current date, so the UI never has to trust the client
    /// clock when defaulting invoice or payment dates.
    pub today: String,
    #[ts(as = "f64")]
    pub student_count: i64,
    #[ts(as = "f64")]
    pub invoice_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct DatabaseHealth {
    pub checked_at: String,
    pub schema_version: i32,
    pub expected_schema_version: i32,
    pub integrity_check: String,
    pub foreign_keys_ok: bool,
    pub backup_count: usize,
    pub latest_backup_path: Option<String>,
    pub latest_backup_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct MoveStudentInput {
    pub student_id: String,
    pub term_id: String,
    pub class_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PromotionEntryInput {
    pub student_id: String,
    pub outcome: String,
    #[serde(default)]
    pub to_class_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct ApplyPromotionInput {
    pub from_year: i32,
    pub to_year: i32,
    pub entries: Vec<PromotionEntryInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PromotionCandidate {
    pub student_id: String,
    pub student_number: String,
    pub first_name: String,
    pub last_name: String,
    pub from_class_id: String,
    pub from_class_name: String,
    pub from_grade_level: i32,
    pub from_stream: String,
    pub suggested_outcome: String,
    pub suggested_class_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PromotionPreview {
    pub from_year: i32,
    pub to_year: i32,
    pub term_ready: bool,
    pub candidates: Vec<PromotionCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PromotionResult {
    pub promoted: i32,
    pub repeated: i32,
    pub graduated: i32,
}
