use crate::domain::entities::Student;
use crate::domain::errors::AppResult;
use crate::domain::ports::StudentRepository;
use rusqlite::{params, Connection};

pub struct SqliteStudentRepository<'c> {
    conn: &'c Connection,
}

impl<'c> SqliteStudentRepository<'c> {
    pub fn new(conn: &'c Connection) -> Self {
        Self { conn }
    }
}

const STUDENT_COLUMNS: &str = "id, student_number, first_name, last_name, dob, gender, \
     admission_date, guardian_name, guardian_phone, guardian_email, status, created_at";

fn map_student(row: &rusqlite::Row<'_>) -> rusqlite::Result<Student> {
    Ok(Student {
        id: row.get(0)?,
        student_number: row.get(1)?,
        first_name: row.get(2)?,
        last_name: row.get(3)?,
        dob: row.get(4)?,
        gender: row.get(5)?,
        admission_date: row.get(6)?,
        guardian_name: row.get(7)?,
        guardian_phone: row.get(8)?,
        guardian_email: row.get(9)?,
        status: row.get(10)?,
        created_at: row.get(11)?,
    })
}

impl StudentRepository for SqliteStudentRepository<'_> {
    fn create(&self, student: &Student) -> AppResult<()> {
        self.conn.execute(
            "INSERT INTO students (id, student_number, first_name, last_name, dob, gender, \
             admission_date, guardian_name, guardian_phone, guardian_email, status) \
             VALUES (?,?,?,?,?,?,?,?,?,?,?)",
            params![
                student.id,
                student.student_number,
                student.first_name,
                student.last_name,
                student.dob,
                student.gender,
                student.admission_date,
                student.guardian_name,
                student.guardian_phone,
                student.guardian_email,
                student.status,
            ],
        )?;
        Ok(())
    }

    fn find_by_id(&self, id: &str) -> AppResult<Option<Student>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM students WHERE id = ?",
            STUDENT_COLUMNS
        ))?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_student(row)?)),
            None => Ok(None),
        }
    }

    fn list(&self) -> AppResult<Vec<Student>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM students ORDER BY last_name, first_name",
            STUDENT_COLUMNS
        ))?;
        let rows = stmt.query_map([], map_student)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn list_by_status(&self, status: &str) -> AppResult<Vec<Student>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {} FROM students WHERE status = ? ORDER BY last_name, first_name",
            STUDENT_COLUMNS
        ))?;
        let rows = stmt.query_map(params![status], map_student)?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    fn update_status(&self, id: &str, status: &str) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE students SET status = ? WHERE id = ?",
            params![status, id],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found("Student", id));
        }
        Ok(())
    }

    fn update_details(&self, student: &Student) -> AppResult<()> {
        let changed = self.conn.execute(
            "UPDATE students SET first_name = ?, last_name = ?, dob = ?, gender = ?, \
             admission_date = ?, guardian_name = ?, guardian_phone = ?, guardian_email = ? \
             WHERE id = ?",
            params![
                student.first_name,
                student.last_name,
                student.dob,
                student.gender,
                student.admission_date,
                student.guardian_name,
                student.guardian_phone,
                student.guardian_email,
                student.id,
            ],
        )?;
        if changed == 0 {
            return Err(crate::domain::errors::AppError::not_found(
                "Student",
                &student.id,
            ));
        }
        Ok(())
    }
}
