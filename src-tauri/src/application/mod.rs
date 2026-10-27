pub mod audit_service;
pub mod auth_service;
pub mod billing_service;
pub mod credit_service;
pub mod discount_calculator;
pub mod discount_service;
pub mod enrollment_service;
pub mod fee_structure_service;
pub mod payment_service;
pub mod promotion_service;
pub mod report_service;
pub mod settings_service;
pub mod student_service;

use chrono::{Datelike, Utc};

/// The current calendar year, used to build document numbers.
pub fn current_year() -> i32 {
    Utc::now().year()
}

/// Today's date in `YYYY-MM-DD`.
pub fn today() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// The current timestamp in ISO-8601.
pub fn now_iso() -> String {
    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}
