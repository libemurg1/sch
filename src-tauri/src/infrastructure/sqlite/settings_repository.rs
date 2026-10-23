use crate::domain::auth::SchoolSettings;
use crate::domain::errors::AppResult;
use crate::domain::ports::SettingsRepository;
use rusqlite::{params, Connection};
use std::collections::HashMap;

pub struct SqliteSettingsRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteSettingsRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const KEY_SCHOOL_NAME: &str = "school.name";
const KEY_LOGO: &str = "school.logo";
const KEY_ADDRESS: &str = "school.address";
const KEY_PHONE: &str = "school.phone";
const KEY_EMAIL: &str = "school.email";
const KEY_CURRENCY_CODE: &str = "currency.code";
const KEY_CURRENCY_SYMBOL: &str = "currency.symbol";
const KEY_BANK_NAME: &str = "bank.name";
const KEY_BANK_ACCOUNT_NAME: &str = "bank.account_name";
const KEY_BANK_ACCOUNT_NUMBER: &str = "bank.account_number";
const KEY_BANK_BRANCH: &str = "bank.branch";
const KEY_BANK_SWIFT: &str = "bank.swift";
const KEY_RECEIPT_FOOTER: &str = "footer.receipt";
const KEY_INVOICE_FOOTER: &str = "footer.invoice";

pub const DEFAULT_SCHOOL_NAME: &str = "School Fees System";
pub const DEFAULT_CURRENCY_CODE: &str = "USD";
pub const DEFAULT_CURRENCY_SYMBOL: &str = "$";

fn load(conn: &Connection) -> AppResult<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT key, value FROM school_settings")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for row in rows {
        let (key, value) = row?;
        map.insert(key, value);
    }
    Ok(map)
}

fn value(map: &HashMap<String, String>, key: &str) -> Option<String> {
    map.get(key).filter(|v| !v.is_empty()).cloned()
}

fn upsert(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO school_settings (key, value) VALUES (?, ?) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

fn upsert_optional(conn: &Connection, key: &str, value: &Option<String>) -> AppResult<()> {
    upsert(conn, key, value.as_deref().unwrap_or(""))
}

impl SettingsRepository for SqliteSettingsRepository<'_> {
    fn get(&self) -> AppResult<SchoolSettings> {
        let map = load(self.conn)?;
        Ok(SchoolSettings {
            school_name: value(&map, KEY_SCHOOL_NAME).unwrap_or_else(|| DEFAULT_SCHOOL_NAME.into()),
            logo_data: value(&map, KEY_LOGO),
            address: value(&map, KEY_ADDRESS),
            phone: value(&map, KEY_PHONE),
            email: value(&map, KEY_EMAIL),
            currency_code: value(&map, KEY_CURRENCY_CODE)
                .unwrap_or_else(|| DEFAULT_CURRENCY_CODE.into()),
            currency_symbol: value(&map, KEY_CURRENCY_SYMBOL)
                .unwrap_or_else(|| DEFAULT_CURRENCY_SYMBOL.into()),
            bank_name: value(&map, KEY_BANK_NAME),
            bank_account_name: value(&map, KEY_BANK_ACCOUNT_NAME),
            bank_account_number: value(&map, KEY_BANK_ACCOUNT_NUMBER),
            bank_branch: value(&map, KEY_BANK_BRANCH),
            bank_swift: value(&map, KEY_BANK_SWIFT),
            receipt_footer: value(&map, KEY_RECEIPT_FOOTER),
            invoice_footer: value(&map, KEY_INVOICE_FOOTER),
        })
    }

    fn save(&self, settings: &SchoolSettings) -> AppResult<()> {
        upsert(self.conn, KEY_SCHOOL_NAME, &settings.school_name)?;
        upsert_optional(self.conn, KEY_ADDRESS, &settings.address)?;
        upsert_optional(self.conn, KEY_PHONE, &settings.phone)?;
        upsert_optional(self.conn, KEY_EMAIL, &settings.email)?;
        upsert(self.conn, KEY_CURRENCY_CODE, &settings.currency_code)?;
        upsert(self.conn, KEY_CURRENCY_SYMBOL, &settings.currency_symbol)?;
        upsert_optional(self.conn, KEY_BANK_NAME, &settings.bank_name)?;
        upsert_optional(
            self.conn,
            KEY_BANK_ACCOUNT_NAME,
            &settings.bank_account_name,
        )?;
        upsert_optional(
            self.conn,
            KEY_BANK_ACCOUNT_NUMBER,
            &settings.bank_account_number,
        )?;
        upsert_optional(self.conn, KEY_BANK_BRANCH, &settings.bank_branch)?;
        upsert_optional(self.conn, KEY_BANK_SWIFT, &settings.bank_swift)?;
        upsert_optional(self.conn, KEY_RECEIPT_FOOTER, &settings.receipt_footer)?;
        upsert_optional(self.conn, KEY_INVOICE_FOOTER, &settings.invoice_footer)?;
        Ok(())
    }

    fn save_logo(&self, logo_data: Option<&str>) -> AppResult<()> {
        match logo_data {
            Some(data) => upsert(self.conn, KEY_LOGO, data),
            None => {
                self.conn
                    .execute("DELETE FROM school_settings WHERE key = ?", params![KEY_LOGO])?;
                Ok(())
            }
        }
    }
}
