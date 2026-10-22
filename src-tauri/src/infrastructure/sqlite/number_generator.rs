use crate::domain::errors::AppError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub struct NumberGenerator {
    conn: Arc<Mutex<Connection>>,
}

impl NumberGenerator {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
    
    pub fn next(&self, key: &str) -> Result<String, AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO number_counters (key, last_value) VALUES (?, 0)",
            [key],
        )?;
        conn.execute(
            "UPDATE number_counters SET last_value = last_value + 1 WHERE key = ?",
            [key],
        )?;
        let last: i64 = conn.query_row(
            "SELECT last_value FROM number_counters WHERE key = ?",
            [key],
            |row| row.get(0),
        )?;
        Ok(last.to_string())
    }
    
    pub fn next_formatted(&self, prefix: &str, year: i32, width: usize) -> Result<String, AppError> {
        let key = format!("{}-{}", prefix, year);
        let seq = self.next(&key)?;
        let seq_num: i64 = seq.parse().unwrap_or(0);
        Ok(format!("{}-{}-{:0width$}", prefix, year, seq_num, width = width))
    }
}
