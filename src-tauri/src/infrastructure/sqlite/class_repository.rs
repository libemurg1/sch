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

    fn find_by_name(&self, name: &str) -> AppResult<Option<Class>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, grade_level, capacity FROM classes WHERE name = ?",
        )?;
        let mut rows = stmt.query(params![name])?;
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

    fn update(&self, class: &Class) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE classes SET name = ?, grade_level = ?, capacity = ? WHERE id = ?",
            params![
                class.name,
                class.grade_level,
                class.capacity,
                class.id
            ],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "Class", &class.id,
            ));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> AppResult<()> {
        let changed = self
            .conn
            .execute("DELETE FROM classes WHERE id = ?", params![id])?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "Class", id,
            ));
        }
        Ok(())
    }

    fn reference_count(&self, id: &str) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT \
               (SELECT COUNT(*) FROM enrollments WHERE class_id = ?1) \
             + (SELECT COUNT(*) FROM fee_structures WHERE class_id = ?1) \
             + (SELECT COUNT(*) FROM invoices i \
                  JOIN fee_structures fs ON fs.id = i.fee_structure_id \
                 WHERE fs.class_id = ?1)",
            params![id],
            |row| row.get(0),
        )?)
    }
}
