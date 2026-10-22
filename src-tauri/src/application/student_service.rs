use crate::domain::entities::*;
use crate::domain::errors::AppResult;
use crate::domain::ports::*;
use uuid::Uuid;

pub struct StudentService {
    students: std::sync::Arc<dyn StudentRepository>,
    numbers: std::sync::Arc<crate::infrastructure::sqlite::number_generator::NumberGenerator>,
}

impl StudentService {
    pub fn new(students: std::sync::Arc<dyn StudentRepository>, numbers: std::sync::Arc<crate::infrastructure::sqlite::number_generator::NumberGenerator>) -> Self {
        Self { students, numbers }
    }
    
    pub fn register(&self, first_name: String, last_name: String, dob: String, gender: Option<String>, admission_date: String, guardian_name: Option<String>, guardian_phone: Option<String>, guardian_email: Option<String>) -> AppResult<Student> {
        if first_name.trim().is_empty() || last_name.trim().is_empty() || dob.trim().is_empty() {
            return Err(crate::domain::errors::AppError { code: "VALIDATION".to_string(), message: "Required fields missing".to_string() });
        }
        let year: i32 = chrono::Utc::now().format("%Y").to_string().parse().unwrap_or(2026);
        let seq = self.numbers.next_formatted("STU", year, 4)?;
        let student = Student {
            id: Uuid::new_v4().to_string(),
            student_number: seq,
            first_name,
            last_name,
            dob,
            gender,
            admission_date,
            guardian_name,
            guardian_phone,
            guardian_email,
            status: "ACTIVE".to_string(),
            created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        };
        self.students.create(&student)?;
        Ok(student)
    }
    
    pub fn list(&self) -> AppResult<Vec<Student>> {
        self.students.list()
    }
}
