mod application;
mod commands;
mod domain;
mod infrastructure;
mod license;

use domain::auth::{Permission, User};
use domain::ports::UserRepository;
use infrastructure::db::Db;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::Manager;

/// Tracks brute-force attempts per username.
#[derive(Debug, Default)]
pub struct LoginGuard {
    attempts: HashMap<String, (u32, Option<Instant>)>,
}

impl LoginGuard {
    fn check(&self, username: &str) -> Result<(), crate::domain::errors::AppError> {
        let key = username.trim().to_lowercase();
        if let Some((count, Some(until))) = self.attempts.get(&key) {
            if *count >= 5 && Instant::now() < *until {
                let remaining = until.saturating_duration_since(Instant::now());
                let seconds = remaining.as_secs().max(1);
                return Err(crate::domain::errors::AppError::new(
                    "RATE_LIMITED",
                    format!(
                        "Too many failed attempts. Try again in {} second{}.",
                        seconds,
                        if seconds == 1 { "" } else { "s" }
                    ),
                ));
            }
        }
        Ok(())
    }

    fn record_failure(&mut self, username: &str) {
        let key = username.trim().to_lowercase();
        let entry = self.attempts.entry(key).or_insert((0, None));
        entry.0 += 1;
        if entry.0 >= 5 {
            entry.1 = Some(Instant::now() + Duration::from_secs(60));
        }
    }

    fn record_success(&mut self, username: &str) {
        let key = username.trim().to_lowercase();
        self.attempts.remove(&key);
    }
}

/// State shared with every command handler.
pub struct AppState {
    pub db: Arc<Db>,
    pub db_path: PathBuf,
    pub remembered_session_path: PathBuf,
    pub licensing: license::Licensing,
    /// The currently authenticated user for this app instance. Cleared on logout.
    pub session: Mutex<Option<User>>,
    pub remembered_session_token: Mutex<Option<String>>,
    pub session_restore_notice: Mutex<Option<String>>,
    pub login_guard: Mutex<LoginGuard>,
}

impl AppState {
    /// Returns the authenticated user, or `UNAUTHORIZED` if nobody is logged in.
    pub fn require_auth(&self) -> crate::domain::errors::AppResult<User> {
        self.session
            .lock()
            .map_err(|_| {
                crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
            })?
            .clone()
            .ok_or_else(crate::domain::errors::AppError::unauthorized)
    }

    pub fn require_write_auth(&self) -> crate::domain::errors::AppResult<User> {
        self.licensing.ensure_writable()?;
        self.require_auth()
    }

    pub fn require_permission(
        &self,
        permission: Permission,
    ) -> crate::domain::errors::AppResult<User> {
        let session_user = self.require_auth()?;
        let user = self.db.with_conn(|conn| {
            crate::infrastructure::sqlite::user_repository::SqliteUserRepository::new(conn)
                .find_by_id(&session_user.id)
        })?;
        let user = user.ok_or_else(crate::domain::errors::AppError::unauthorized)?;
        if !user.is_active {
            return Err(crate::domain::errors::AppError::forbidden(
                "This account has been deactivated",
            ));
        }
        if !user.role.allows(permission) {
            return Err(crate::domain::errors::AppError::forbidden(
                "Your account does not have permission to perform this action",
            ));
        }
        Ok(user)
    }

    pub fn require_write_permission(
        &self,
        permission: Permission,
    ) -> crate::domain::errors::AppResult<User> {
        self.licensing.ensure_writable()?;
        self.require_permission(permission)
    }

    /// Returns the authenticated user if logged in, without failing.
    pub fn current_user(&self) -> crate::domain::errors::AppResult<Option<User>> {
        Ok(self
            .session
            .lock()
            .map_err(|_| {
                crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
            })?
            .clone())
    }

    pub fn take_session_restore_notice(&self) -> crate::domain::errors::AppResult<Option<String>> {
        self.session_restore_notice
            .lock()
            .map_err(|_| {
                crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
            })
            .map(|mut notice| notice.take())
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;

            let db = Db::open(&app_dir)
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let licensing = license::Licensing::open(&app_dir)
                .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            let remembered_session_path = app_dir.join("remembered-session.bin");
            let (restored_user, remembered_session_token, mut session_restore_notice) =
                restore_remembered_session(&db, &remembered_session_path)
                    .map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })?;
            if let Some(database_notice) = db.startup_recovery_notice() {
                session_restore_notice = Some(match session_restore_notice {
                    Some(session_notice) => format!("{database_notice} {session_notice}"),
                    None => database_notice,
                });
            }

            app.manage(AppState {
                db: Arc::new(db),
                db_path: app_dir.join("school.db"),
                remembered_session_path,
                licensing,
                session: Mutex::new(restored_user),
                remembered_session_token: Mutex::new(remembered_session_token),
                session_restore_notice: Mutex::new(session_restore_notice),
                login_guard: Mutex::new(LoginGuard::default()),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::get_auth_status,
            commands::auth::login,
            commands::auth::logout,
            commands::auth::get_current_user,
            commands::auth::create_user,
            commands::auth::list_users,
            commands::auth::change_password,
            commands::auth::create_administrator_recovery_code,
            commands::auth::administrator_recovery_configured,
            commands::auth::recover_administrator,
            commands::auth::set_user_active,
            commands::auth::update_user,
            commands::auth::delete_user,
            commands::printing::show_print_dialog,
            commands::support::open_developer_profile,
            commands::settings::get_school_settings,
            commands::settings::update_school_settings,
            commands::settings::set_logo,
            commands::students::list_students,
            commands::students::list_students_by_status,
            commands::students::register_student,
            commands::students::import_students,
            commands::students::get_student,
            commands::students::update_student,
            commands::students::update_student_status,
            commands::enrollment::list_classes,
            commands::enrollment::list_grades,
            commands::enrollment::create_class,
            commands::enrollment::update_class,
            commands::enrollment::delete_class,
            commands::enrollment::list_terms,
            commands::enrollment::get_active_term,
            commands::enrollment::create_term,
            commands::enrollment::update_term,
            commands::enrollment::delete_term,
            commands::enrollment::enroll_student,
            commands::enrollment::move_student,
            commands::enrollment::list_enrollments,
            commands::enrollment::list_enrollments_by_term,
            commands::enrollment::list_student_enrollments,
            commands::promotion::copy_classes,
            commands::promotion::preview_promotion,
            commands::promotion::apply_promotion,
            commands::fee_structures::get_fee_structure,
            commands::fee_structures::save_fee_structure,
            commands::billing::generate_invoice_batch,
            commands::billing::cancel_invoice,
            commands::billing::generate_supplementary_invoice,
            commands::billing::assign_student_fee,
            commands::billing::list_invoices,
            commands::billing::list_student_invoices,
            commands::billing::list_class_invoices,
            commands::billing::get_invoice,
            commands::billing::get_invoice_items,
            commands::payments::record_payment,
            commands::payments::list_payments,
            commands::payments::list_student_payments,
            commands::payments::get_payment,
            commands::payments::get_payment_allocations,
            commands::payments::get_student_account,
            commands::payments::reverse_payment,
            commands::discounts::list_discounts,
            commands::discounts::create_discount,
            commands::discounts::list_student_discounts,
            commands::discounts::assign_discount,
            commands::discounts::list_all_student_discounts,
            commands::discounts::revoke_discount,
            commands::statements::get_student_statement,
            commands::statements::get_invoice_detail,
            commands::setup::get_app_info,
            commands::setup::get_database_health,
            commands::setup::backup_database,
            commands::setup::list_backups,
            commands::setup::restore_database,
            commands::setup::export_backup,
            commands::license::get_license_status,
            commands::license::activate_license,
            commands::reports::get_finance_report,
            commands::reports::list_audit_entries,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the School System application");
}

fn restore_remembered_session(
    db: &Db,
    path: &std::path::Path,
) -> crate::domain::errors::AppResult<(Option<User>, Option<String>, Option<String>)> {
    let token = match infrastructure::remembered_session::load(path) {
        Ok(token) => token,
        Err(error)
            if matches!(
                error.code.as_str(),
                "KEY_UNPROTECTION_FAILED" | "SESSION_RESTORE_FAILED"
            ) =>
        {
            infrastructure::remembered_session::clear(path)?;
            return Ok((
                None,
                None,
                Some("The saved sign-in could not be restored. Please sign in again.".into()),
            ));
        }
        Err(error) => return Err(error),
    };
    let Some(token) = token else {
        return Ok((None, None, None));
    };
    match application::auth_service::AuthService::restore_remembered_session(db, &token)? {
        Some(user) => Ok((Some(user), Some(token), None)),
        None => {
            infrastructure::remembered_session::clear(path)?;
            Ok((
                None,
                None,
                Some("The saved sign-in is no longer valid. Please sign in again.".into()),
            ))
        }
    }
}
