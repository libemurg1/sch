use crate::domain::entities::Class;
use crate::domain::errors::AppResult;
use crate::domain::ports::ClassRepository;
use rusqlite::{params, Connection};

pub struct SqliteClassRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteClassRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

fn map_class(row: &rusqlite::Row<'_>) -> rusqlite::Result<Class> {
    Ok(Class {
        id: row.get(0)?,
        name: row.get(1)?,
        grade_level: row.get(2)?,
        capacity: row.get(3)?,
    })
}

impl ClassRepository for SqliteClassRepository<'_> {
    fn list(&self) -> AppResult<Vec<Class>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, grade_level, capacity FROM classes ORDER BY grade_level, name")?;
        let rows = stmt.query_map([], map_class)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Class>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, grade_level, capacity FROM classes WHERE id = ?",
        )?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_class(row)?)),
            None => Ok(None),
        }
    }

    fn create(&self, class: &Class) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO classes (id, name, grade_level, capacity) VALUES (?,?,?,?)",
            params![class.id, class.name, class.grade_level, class.capacity],
        )?;
        Ok(())
    }
}
