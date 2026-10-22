use crate::domain::entities::*;
use crate::domain::errors::AppResult;
use crate::domain::ports::*;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

pub struct SqliteEnrollmentRepository {
    conn: Arc<Mutex<Connection>>,
}
impl SqliteEnrollmentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self { Self { conn } }
}
impl EnrollmentRepository for SqliteEnrollmentRepository {
    fn create(&self, enrollment: &Enrollment) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO enrollments (id, student_id, class_id, term_id, status) VALUES (?,?,?,?,?)",
            params![enrollment.id, enrollment.student_id, enrollment.class_id, enrollment.term_id, enrollment.status],
        )?;
        Ok(())
    }
    fn find_by_student_term(&self, student_id: &str, term_id: &str) -> AppResult<Option<Enrollment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, student_id, class_id, term_id, status, enrolled_at FROM enrollments WHERE student_id = ? AND term_id = ?")?;
        let mut rows = stmt.query(params![student_id, term_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Enrollment { id: row.get(0)?, student_id: row.get(1)?, class_id: row.get(2)?, term_id: row.get(3)?, status: row.get(4)?, enrolled_at: row.get(5)? }))
        } else { Ok(None) }
    }
    fn count_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<i32> {
        let conn = self.conn.lock().unwrap();
        Ok(conn.query_row("SELECT COUNT(*) FROM enrollments WHERE class_id = ? AND term_id = ? AND status = 'ACTIVE'", params![class_id, term_id], |r| r.get(0))?)
    }
    fn list_by_class_term(&self, class_id: &str, term_id: &str) -> AppResult<Vec<Enrollment>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, student_id, class_id, term_id, status, enrolled_at FROM enrollments WHERE class_id = ? AND term_id = ? AND status = 'ACTIVE'")?;
        let rows = stmt.query_map(params![class_id, term_id], |row| {
            Ok(Enrollment { id: row.get(0)?, student_id: row.get(1)?, class_id: row.get(2)?, term_id: row.get(3)?, status: row.get(4)?, enrolled_at: row.get(5)? })
        })?;
        let mut res = Vec::new();
        for r in rows { res.push(r?); }
        Ok(res)
    }
}
