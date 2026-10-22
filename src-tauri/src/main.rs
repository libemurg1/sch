mod application;
mod commands;
mod domain;
mod infrastructure;

use infrastructure::db::Db;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

/// State shared with every command handler.
pub struct AppState {
    pub db: Arc<Db>,
    pub db_path: PathBuf,
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

            app.manage(AppState {
                db: Arc::new(db),
                db_path: app_dir.join("school.db"),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::students::list_students,
            commands::students::list_students_by_status,
            commands::students::register_student,
            commands::students::get_student,
            commands::students::update_student_status,
            commands::enrollment::list_classes,
            commands::enrollment::create_class,
            commands::enrollment::list_terms,
            commands::enrollment::get_active_term,
            commands::enrollment::create_term,
            commands::enrollment::enroll_student,
            commands::enrollment::list_enrollments,
            commands::enrollment::list_student_enrollments,
            commands::fee_structures::get_fee_structure,
            commands::fee_structures::save_fee_structure,
            commands::billing::generate_invoice_batch,
            commands::billing::generate_supplementary_invoice,
            commands::billing::assign_student_fee,
            commands::billing::list_invoices,
            commands::billing::list_student_invoices,
            commands::billing::list_class_invoices,
            commands::billing::get_invoice,
            commands::billing::get_invoice_items,
            commands::payments::record_payment,
            commands::payments::list_student_payments,
            commands::payments::get_payment,
            commands::payments::get_payment_allocations,
            commands::discounts::list_discounts,
            commands::discounts::create_discount,
            commands::discounts::list_student_discounts,
            commands::discounts::assign_discount,
            commands::statements::get_student_statement,
            commands::statements::get_invoice_detail,
            commands::setup::get_app_info,
            commands::setup::backup_database,
            commands::setup::list_backups,
            commands::setup::restore_database,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the School Fees System application");
}
