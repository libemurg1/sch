use crate::application::{current_year, now_iso};
use crate::domain::dto::RegisterStudentInput;
use crate::domain::entities::Student;
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::StudentRepository;
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::number_generator::NumberGenerator;
use crate::infrastructure::sqlite::student_repository::SqliteStudentRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

/// Use cases for registering and listing students.
pub struct StudentService;

impl StudentService {
    /// Registers a new student, allocating the next `STU-YYYY-NNNN` number.
    ///
    /// The number allocation and the insert happen in one transaction so a
    /// rolled back registration never burns a number.
    pub fn register(db: &Db, input: RegisterStudentInput) -> AppResult<Student> {
        if input.first_name.trim().is_empty() {
            return Err(AppError::validation("First name is required"));
        }
        if input.last_name.trim().is_empty() {
            return Err(AppError::validation("Last name is required"));
        }
        if input.dob.trim().is_empty() {
            return Err(AppError::validation("Date of birth is required"));
        }
        if input.admission_date.trim().is_empty() {
            return Err(AppError::validation("Admission date is required"));
        }

        db.execute(|conn| {
            let students = SqliteStudentRepository::new(conn);
            let numbers = NumberGenerator::new(conn);

            let student_number = numbers.next_formatted("STU", current_year(), 4)?;

            let student = Student {
                id: Uuid::new_v4().to_string(),
                student_number,
                first_name: input.first_name.trim().to_string(),
                last_name: input.last_name.trim().to_string(),
                dob: input.dob.trim().to_string(),
                gender: input.gender,
                admission_date: input.admission_date.trim().to_string(),
                guardian_name: input.guardian_name,
                guardian_phone: input.guardian_phone,
                guardian_email: input.guardian_email,
                status: "ACTIVE".to_string(),
                created_at: now_iso(),
            };

            students.create(&student)?;
            Ok(student)
        })
    }

    pub fn list(db: &Db) -> AppResult<Vec<Student>> {
        db.with_conn(|conn| SqliteStudentRepository::new(conn).list())
    }

    pub fn list_by_status(db: &Db, status: &str) -> AppResult<Vec<Student>> {
        db.with_conn(|conn| SqliteStudentRepository::new(conn).list_by_status(status))
    }

    pub fn find(db: &Db, id: &str) -> AppResult<Option<Student>> {
        db.with_conn(|conn| SqliteStudentRepository::new(conn).find_by_id(id))
    }

    /// Soft-state change: students are never hard deleted.
    pub fn update_status(db: &Db, id: &str, status: &str) -> AppResult<()> {
        const ALLOWED: [&str; 3] = ["ACTIVE", "INACTIVE", "GRADUATED"];
        if !ALLOWED.contains(&status) {
            return Err(AppError::validation(format!(
                "Status must be one of {:?}",
                ALLOWED
            )));
        }

        db.execute(|conn| {
            let students = SqliteStudentRepository::new(conn);
            students.update_status(id, status)
        })
    }
}
