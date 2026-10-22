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
pub struct CreateClassInput {
    pub name: String,
    pub grade_level: i32,
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
pub struct PaymentAllocationInput {
    pub invoice_id: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
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
    pub allocations: Vec<PaymentAllocationInput>,
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
    #[ts(as = "f64")]
    pub total_billed_cents: i64,
    #[ts(as = "f64")]
    pub total_paid_cents: i64,
    #[ts(as = "f64")]
    pub balance_cents: i64,
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
