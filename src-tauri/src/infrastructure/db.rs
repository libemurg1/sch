use crate::domain::errors::AppError;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct DbState {
    pub conn: Arc<Mutex<Connection>>,
}

impl DbState {
    pub fn new(app_dir: &PathBuf) -> Result<Self, AppError> {
        fs::create_dir_all(app_dir)?;
        let db_path = app_dir.join("school.db");
        
        // Backup before any migration if db exists
        if db_path.exists() {
            let backup_path = app_dir.join(format!(
                "school_backup_{}.db",
                chrono::Utc::now().format("%Y%m%d_%H%M%S")
            ));
            fs::copy(&db_path, &backup_path)?;
        }
        
        let conn = Connection::open(&db_path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA busy_timeout = 5000;")?;
        
        let db_state = DbState {
            conn: Arc::new(Mutex::new(conn)),
        };
        
        db_state.run_migrations()?;
        Ok(db_state)
    }
    
    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let user_version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        
        let mut migrations_dir = std::env::current_dir()?;
        migrations_dir.push("migrations");
        
        // Also try src-tauri/migrations
        if !migrations_dir.exists() {
            migrations_dir = std::env::current_dir()?;
            migrations_dir.push("src-tauri");
            migrations_dir.push("migrations");
        }
        
        if !migrations_dir.exists() {
            return Ok(());
        }
        
        let mut entries: Vec<_> = fs::read_dir(&migrations_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());
        
        let mut applied = user_version;
        for entry in entries {
            let version = applied + 1;
            let sql = fs::read_to_string(entry.path())?;
            conn.execute_batch(&sql)?;
            conn.execute(&format!("PRAGMA user_version = {}", version), [])?;
            applied = version;
        }
        
        Ok(())
    }
}
