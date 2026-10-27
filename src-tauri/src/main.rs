mod application;
mod commands;
mod domain;
mod infrastructure;
mod license;

use domain::auth::User;
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
    pub licensing: license::Licensing,
    /// The currently authenticated user for this app instance. Cleared on logout.
    pub session: Mutex<Option<User>>,
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

            app.manage(AppState {
                db: Arc::new(db),
                db_path: app_dir.join("school.db"),
                licensing,
                session: Mutex::new(None),
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
