use crate::domain::entities::{Class, Grade};
use crate::domain::errors::{AppError, AppResult};
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

const CLASS_COLUMNS: &str = "id, name, grade_level, stream, academic_year, capacity";

fn map_class(row: &rusqlite::Row<'_>) -> rusqlite::Result<Class> {
    Ok(Class {
        id: row.get(0)?,
        name: row.get(1)?,
        grade_level: row.get(2)?,
        stream: row.get(3)?,
        academic_year: row.get(4)?,
        capacity: row.get(5)?,
    })
}

fn map_grade(row: &rusqlite::Row<'_>) -> rusqlite::Result<Grade> {
    Ok(Grade {
        level: row.get(0)?,
        name: row.get(1)?,
        sort_order: row.get(2)?,
    })
}

impl ClassRepository for SqliteClassRepository<'_> {
    fn list(&self) -> AppResult<Vec<Class>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {CLASS_COLUMNS} FROM classes \
             ORDER BY academic_year DESC, grade_level, stream, name"
        ))?;
        let rows = stmt.query_map([], map_class)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_year(&self, year: i32) -> AppResult<Vec<Class>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {CLASS_COLUMNS} FROM classes WHERE academic_year = ? \
             ORDER BY grade_level, stream, name"
        ))?;
        let rows = stmt.query_map(params![year], map_class)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_grades(&self) -> AppResult<Vec<Grade>> {
        let mut stmt = self
            .conn
            .prepare("SELECT level, name, sort_order FROM grades ORDER BY sort_order")?;
        let rows = stmt.query_map([], map_grade)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn find_grade(&self, level: i32) -> AppResult<Option<Grade>> {
        let mut stmt = self
            .conn
            .prepare("SELECT level, name, sort_order FROM grades WHERE level = ?")?;
        let mut rows = stmt.query(params![level])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_grade(row)?)),
            None => Ok(None),
        }
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Class>> {
        let mut stmt = self
            .conn
            .prepare(&format!("SELECT {CLASS_COLUMNS} FROM classes WHERE id = ?"))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_class(row)?)),
            None => Ok(None),
        }
    }

    fn find_by_year_name(&self, year: i32, name: &str) -> AppResult<Option<Class>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {CLASS_COLUMNS} FROM classes WHERE academic_year = ? AND name = ?"
        ))?;
        let mut rows = stmt.query(params![year, name])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_class(row)?)),
            None => Ok(None),
        }
    }

    fn create(&self, class: &Class) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO classes (id, name, grade_level, stream, academic_year, capacity) \
             VALUES (?,?,?,?,?,?)",
            params![
                class.id,
                class.name,
                class.grade_level,
                class.stream,
                class.academic_year,
                class.capacity
            ],
        )?;
        Ok(())
    }

    fn update(&self, class: &Class) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE classes SET name = ?, grade_level = ?, stream = ?, academic_year = ?, \
             capacity = ? WHERE id = ?",
            params![
                class.name,
                class.grade_level,
                class.stream,
                class.academic_year,
                class.capacity,
                class.id
            ],
        )?;
        if changed == 0 {
            return Err(AppError::not_found("Class", &class.id));
        }
        Ok(())
    }

    fn delete(&self, id: &str) -> AppResult<()> {
        let changed = self
            .conn
            .execute("DELETE FROM classes WHERE id = ?", params![id])?;
        if changed == 0 {
            return Err(AppError::not_found("Class", id));
        }
        Ok(())
    }

    fn reference_count(&self, id: &str) -> AppResult<i64> {
        Ok(self.conn.query_row(
            "SELECT \
               (SELECT COUNT(*) FROM enrollments WHERE class_id = ?1) \
             + (SELECT COUNT(*) FROM promotions WHERE from_class_id = ?1 OR to_class_id = ?1) \
             + (SELECT COUNT(*) FROM fee_structures WHERE class_id = ?1) \
             + (SELECT COUNT(*) FROM invoices i \
                  JOIN fee_structures fs ON fs.id = i.fee_structure_id \
                 WHERE fs.class_id = ?1)",
            params![id],
            |row| row.get(0),
        )?)
    }
}
