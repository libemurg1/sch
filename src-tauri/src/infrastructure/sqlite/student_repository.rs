use crate::domain::entities::Student;
use crate::domain::errors::AppResult;
use crate::domain::ports::StudentRepository;
use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};

pub struct SqliteStudentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStudentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl StudentRepository for SqliteStudentRepository {
    fn create(&self, student: &Student) -> AppResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO students (id, student_number, first_name, last_name, dob, gender, admission_date, guardian_name, guardian_phone, guardian_email, status) VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            params![student.id, student.student_number, student.first_name, student.last_name, student.dob, student.gender, student.admission_date, student.guardian_name, student.guardian_phone, student.guardian_email, student.status],
        )?;
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Student>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, student_number, first_name, last_name, dob, gender, admission_date, guardian_name, guardian_phone, guardian_email, status, created_at FROM students WHERE id = ?")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Student {
                id: row.get(0)?, student_number: row.get(1)?, first_name: row.get(2)?, last_name: row.get(3)?, dob: row.get(4)?, gender: row.get(5)?, admission_date: row.get(6)?, guardian_name: row.get(7)?, guardian_phone: row.get(8)?, guardian_email: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
            }))
        } else { Ok(None) }
    }

    fn list(&self) -> AppResult<Vec<Student>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, student_number, first_name, last_name, dob, gender, admission_date, guardian_name, guardian_phone, guardian_email, status, created_at FROM students ORDER BY last_name, first_name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Student {
                id: row.get(0)?, student_number: row.get(1)?, first_name: row.get(2)?, last_name: row.get(3)?, dob: row.get(4)?, gender: row.get(5)?, admission_date: row.get(6)?, guardian_name: row.get(7)?, guardian_phone: row.get(8)?, guardian_email: row.get(9)?, status: row.get(10)?, created_at: row.get(11)?,
            })
        })?;
        let mut res = Vec::new();
        for r in rows { res.push(r?); }
        Ok(res)
    }
}
