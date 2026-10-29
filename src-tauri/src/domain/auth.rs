use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(rename_all = "SCREAMING_SNAKE_CASE")]
#[ts(export)]
pub enum UserRole {
    SchoolAdministrator,
    Bursar,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::SchoolAdministrator
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Permission {
    ManageUsers,
    ManageSettings,
    ManageStudents,
    ManageEnrollment,
    ManageFeeStructures,
    ManageDiscounts,
    ManageBilling,
    RecordPayments,
    ReversePayments,
    CancelInvoices,
    ViewFinancialReports,
}

impl UserRole {
    pub fn allows(self, permission: Permission) -> bool {
        match self {
            Self::SchoolAdministrator => true,
            Self::Bursar => matches!(
                permission,
                Permission::ManageEnrollment
                    | Permission::ManageFeeStructures
                    | Permission::ManageDiscounts
                    | Permission::ManageBilling
                    | Permission::RecordPayments
                    | Permission::ViewFinancialReports
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Permission, UserRole};

    #[test]
    fn school_administrator_has_every_permission() {
        let permissions = [
            Permission::ManageUsers,
            Permission::ManageSettings,
            Permission::ManageStudents,
            Permission::ManageEnrollment,
            Permission::ManageFeeStructures,
            Permission::ManageDiscounts,
            Permission::ManageBilling,
            Permission::RecordPayments,
            Permission::ReversePayments,
            Permission::CancelInvoices,
            Permission::ViewFinancialReports,
        ];

        assert!(permissions
            .into_iter()
            .all(|permission| UserRole::SchoolAdministrator.allows(permission)));
    }

    #[test]
    fn bursar_has_operational_finance_and_enrollment_permissions_only() {
        assert!(UserRole::Bursar.allows(Permission::ManageEnrollment));
        assert!(UserRole::Bursar.allows(Permission::ManageFeeStructures));
        assert!(UserRole::Bursar.allows(Permission::ManageDiscounts));
        assert!(UserRole::Bursar.allows(Permission::ManageBilling));
        assert!(UserRole::Bursar.allows(Permission::RecordPayments));
        assert!(UserRole::Bursar.allows(Permission::ViewFinancialReports));

        assert!(!UserRole::Bursar.allows(Permission::ManageUsers));
        assert!(!UserRole::Bursar.allows(Permission::ManageSettings));
        assert!(!UserRole::Bursar.allows(Permission::ManageStudents));
        assert!(!UserRole::Bursar.allows(Permission::ReversePayments));
        assert!(!UserRole::Bursar.allows(Permission::CancelInvoices));
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct User {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: String,
    pub last_login_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct SchoolSettings {
    pub school_name: String,
    /// Data URI (`data:image/png;base64,...`) of the school logo, if set.
    pub logo_data: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub currency_symbol: String,
    pub bank_name: Option<String>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub bank_swift: Option<String>,
    pub receipt_footer: Option<String>,
    pub invoice_footer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub remember_me: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangePasswordInput {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub display_name: String,
    #[serde(default)]
    pub role: UserRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateSchoolSettingsInput {
    pub school_name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub currency_code: String,
    pub currency_symbol: String,
    pub bank_name: Option<String>,
    pub bank_account_name: Option<String>,
    pub bank_account_number: Option<String>,
    pub bank_branch: Option<String>,
    pub bank_swift: Option<String>,
    pub receipt_footer: Option<String>,
    pub invoice_footer: Option<String>,
}

/// Snapshot of the authentication state, polled by the UI on start-up.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
#[ts(export)]
pub struct AuthStatus {
    /// False on a first run, in which case the UI must offer to create an admin.
    pub has_users: bool,
    pub user: Option<User>,
    pub session_restore_notice: Option<String>,
}
