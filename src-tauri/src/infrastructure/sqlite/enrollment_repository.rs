use crate::domain::entities::Enrollment;
use crate::domain::errors::AppResult;
use crate::domain::ports::EnrollmentRepository;
use rusqlite::{params, Connection};

pub struct SqliteEnrollmentRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteEnrollmentRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

fn map_enrollment(row: &rusqlite::Row<'_>) -> rusqlite::Result<Enrollment> {
    Ok(Enrollment {
        id: row.get(0)?,
        student_id: row.get(1)?,
        class_id: row.get(2)?,
        term_id: row.get(3)?,
        status: row.get(4)?,
        enrolled_at: row.get(5)?,
    })
}

const ENROLLMENT_COLUMNS: &str = "id, student_id, class_id, term_id, status, enrolled_at";

impl EnrollmentRepository for SqliteEnrollmentRepository<'_> {
    fn create(&self, enrollment: &Enrollment) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO enrollments (id, student_id, class_id, term_id, status) VALUES (?,?,?,?,?)",
            params![
                enrollment.id,
                enrollment.student_id,
                enrollment.class_id,
                enrollment.term_id,
                enrollment.status,
            ],
        )?;
        Ok(())
    }

    fn find_by_student_term(
        &self,
        student_id: &str,
        term_id: &str,
    ) -> AppResult<Option<Enrollment>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM enrollments WHERE student_id = ? AND term_id = ?",
            ENROLLMENT_COLUMNS
        ))?;
        let mut rows = stmt.query(params![student_id, term_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_enrollment(row)?)),
            None => Ok(None),
        }
    }

    fn count_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<i32> {
        let count = self.conn.query_row(
            "SELECT COUNT(*) FROM enrollments WHERE class_id = ? AND term_id = ? AND status = 'ACTIVE'",
            params![class_id, term_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    fn list_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<Vec<Enrollment>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM enrollments WHERE class_id = ? AND term_id = ? AND status = 'ACTIVE' \
             ORDER BY enrolled_at",
            ENROLLMENT_COLUMNS
        ))?;
        let rows = stmt.query_map(params![class_id, term_id], map_enrollment)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_student(&self, student_id: &str) -> AppResult<Vec<Enrollment>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM enrollments WHERE student_id = ? ORDER BY enrolled_at DESC",
            ENROLLMENT_COLUMNS
        ))?;
        let rows = stmt.query_map(params![student_id], map_enrollment)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}
