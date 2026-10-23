use crate::domain::auth::{SchoolSettings, UpdateSchoolSettingsInput};
use crate::domain::errors::AppResult;
use crate::domain::ports::SettingsRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::settings_repository::SqliteSettingsRepository;

pub struct SettingsService;

impl SettingsService {
    pub fn get(db: &Db) -> AppResult<SchoolSettings> {
        db.with_conn(|conn| SqliteSettingsRepository::new(conn).get())
    }

    pub fn update(db: &Db, input: &UpdateSchoolSettingsInput) -> AppResult<SchoolSettings> {
        db.transaction(|conn| {
            let repo = SqliteSettingsRepository::new(conn);
            let mut settings = repo.get()?;
            settings.school_name = input.school_name.trim().to_string();
            if settings.school_name.is_empty() {
                settings.school_name =
                    crate::infrastructure::sqlite::settings_repository::DEFAULT_SCHOOL_NAME
                        .to_string();
            }
            settings.address = normalize(input.address.as_deref());
            settings.phone = normalize(input.phone.as_deref());
            settings.email = normalize(input.email.as_deref());
            settings.currency_code = input.currency_code.trim().to_uppercase();
            if settings.currency_code.is_empty() {
                settings.currency_code =
                    crate::infrastructure::sqlite::settings_repository::DEFAULT_CURRENCY_CODE
                        .to_string();
            }
            settings.currency_symbol = input.currency_symbol.trim().to_string();
            if settings.currency_symbol.is_empty() {
                settings.currency_symbol =
                    crate::infrastructure::sqlite::settings_repository::DEFAULT_CURRENCY_SYMBOL
                        .to_string();
            }
            settings.bank_name = normalize(input.bank_name.as_deref());
            settings.bank_account_name = normalize(input.bank_account_name.as_deref());
            settings.bank_account_number = normalize(input.bank_account_number.as_deref());
            settings.bank_branch = normalize(input.bank_branch.as_deref());
            settings.bank_swift = normalize(input.bank_swift.as_deref());
            settings.receipt_footer = normalize(input.receipt_footer.as_deref());
            settings.invoice_footer = normalize(input.invoice_footer.as_deref());
            repo.save(&settings)?;
            Ok(settings)
        })
    }

    pub fn set_logo(db: &Db, logo_data: Option<String>) -> AppResult<SchoolSettings> {
        db.transaction(|conn| {
            let repo = SqliteSettingsRepository::new(conn);
            repo.save_logo(logo_data.as_deref())?;
            repo.get()
        })
    }
}

fn normalize(value: Option<&str>) -> Option<String> {
    match value {
        Some(v) => {
            let trimmed = v.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        None => None,
    }
}
