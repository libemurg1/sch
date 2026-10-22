use crate::domain::errors::AppResult;
use rusqlite::{params, Connection};

/// Issues human-readable document numbers (`STU-2026-0001`, `INV-2026-003`,
/// `REC-2026-0004`) from the `number_counters` table.
///
/// The generator borrows the caller's connection, so when it is created
/// inside a transaction the counter increment joins that transaction.
pub struct NumberGenerator<'c> {
    conn: &'c Connection,
}

impl<'c> NumberGenerator<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }

    /// Increments the counter for `key` and returns the new value.
    pub fn next(&self, key: &str) -> AppResult<i64> {
        self.conn.execute(
            "INSERT OR IGNORE INTO number_counters (key, last_value) VALUES (?, 0)",
            params![key],
        )?;
        self.conn.execute(
            "UPDATE number_counters SET last_value = last_value + 1 WHERE key = ?",
            params![key],
        )?;
        let last: i64 = self.conn.query_row(
            "SELECT last_value FROM number_counters WHERE key = ?",
            params![key],
            |row| row.get(0),
        )?;
        Ok(last)
    }

    /// Increments the counter for `prefix`-`year` and formats the value.
    pub fn next_formatted(&self, prefix: &str, year: i32, width: usize) -> AppResult<String> {
        let sequence = self.next(&format!("{}-{}", prefix, year))?;
        Ok(format!("{}-{}-{:0width$}", prefix, year, sequence, width = width))
    }
}
