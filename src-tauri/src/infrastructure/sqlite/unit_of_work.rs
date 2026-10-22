use std::sync::{Arc, Mutex};
use rusqlite::Connection;
use crate::domain::errors::AppError;

pub trait UnitOfWork {
    fn execute<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>;
}

pub struct SqliteUnitOfWork {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteUnitOfWork {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl UnitOfWork for SqliteUnitOfWork {
    fn execute<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("BEGIN IMMEDIATE;")?;
        let result = f(&conn);
        match result {
            Ok(val) => {
                conn.execute_batch("COMMIT;")?;
                Ok(val)
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK;");
                Err(e)
            }
        }
    }
}
