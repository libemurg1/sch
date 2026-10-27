use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Student {
    pub id: String,
    pub student_number: String,
    pub first_name: String,
    pub last_name: String,
    pub dob: String,
    pub gender: Option<String>,
    pub admission_date: String,
    pub guardian_name: Option<String>,
    pub guardian_phone: Option<String>,
    pub guardian_email: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Grade {
    pub level: i32,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Class {
    pub id: String,
    pub name: String,
    pub grade_level: i32,
    pub stream: String,
    pub academic_year: i32,
    pub capacity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Term {
    pub id: String,
    pub year: i32,
    pub term_number: i32,
    pub start_date: String,
    pub end_date: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Enrollment {
    pub id: String,
    pub student_id: String,
    pub class_id: String,
    pub term_id: String,
    pub status: String,
    pub enrolled_at: String,
    pub ended_at: Option<String>,
    pub end_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct FeeItem {
    pub id: String,
    pub fee_structure_id: String,
    pub description: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub discountable: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct FeeStructure {
    pub id: String,
    pub class_id: String,
    pub term_id: String,
    #[ts(as = "f64")]
    pub total_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Discount {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[ts(as = "f64")]
    pub value: i64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct StudentDiscount {
    pub id: String,
    pub student_id: String,
    pub discount_id: String,
    pub term_id: Option<String>,
    pub target_fee_item_id: Option<String>,
    pub target_student_fee_id: Option<String>,
    pub reason: Option<String>,
    pub revoked_at: Option<String>,
    pub revoked_by: Option<String>,
    pub revoke_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct StudentFee {
    pub id: String,
    pub student_id: String,
    pub term_id: String,
    pub description: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub discountable: bool,
    pub status: String,
    pub invoice_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Invoice {
    pub id: String,
    pub invoice_number: String,
    pub kind: String,
    pub student_id: String,
    pub term_id: String,
    pub fee_structure_id: Option<String>,
    pub issue_date: String,
    pub due_date: String,
    pub status: String,
    #[ts(as = "f64")]
    pub subtotal_cents: i64,
    #[ts(as = "f64")]
    pub discount_cents: i64,
    #[ts(as = "f64")]
    pub total_cents: i64,
    #[ts(as = "f64")]
    pub paid_cents: i64,
    pub cancelled_at: Option<String>,
    pub cancelled_by: Option<String>,
    pub cancel_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct InvoiceItem {
    pub id: String,
    pub invoice_id: String,
    pub kind: String,
    pub description: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub discountable: bool,
    pub discount_id: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct Payment {
    pub id: String,
    pub receipt_number: String,
    pub student_id: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub payment_date: String,
    pub payment_method: String,
    pub reference_number: Option<String>,
    pub created_by: Option<String>,
    pub created_at: String,
    pub reversed_at: Option<String>,
    pub reversed_by: Option<String>,
    pub reversal_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct PaymentAllocation {
    pub id: String,
    pub payment_id: String,
    pub invoice_id: String,
    #[ts(as = "f64")]
    pub amount_cents: i64,
    pub reversed_at: Option<String>,
    pub reversal_reason: Option<String>,
}
