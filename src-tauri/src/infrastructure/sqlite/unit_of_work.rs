use crate::domain::errors::AppResult;
use rusqlite::Connection;

/// Runs a unit of work against the shared database connection.
///
/// Implemented for [`crate::infrastructure::db::Db`]. The closure receives the
/// connection directly, so repositories called inside it reuse the open
/// transaction instead of opening a second connection or re-acquiring the
/// lock.
pub trait UnitOfWork {
    fn execute<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>;
}

impl UnitOfWork for crate::infrastructure::db::Db {
    fn execute<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        self.transaction(f)
    }
}
