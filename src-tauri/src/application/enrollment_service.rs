use crate::domain::dto::{CreateClassInput, CreateTermInput, EnrollStudentInput};
use crate::domain::entities::{Class, Enrollment, Term};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::{ClassRepository, EnrollmentRepository, TermRepository};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::class_repository::SqliteClassRepository;
use crate::infrastructure::sqlite::enrollment_repository::SqliteEnrollmentRepository;
use crate::infrastructure::sqlite::term_repository::SqliteTermRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use uuid::Uuid;

/// Use cases for classes, terms and enrollments.
pub struct EnrollmentService;

impl EnrollmentService {
    pub fn list_classes(db: &Db) -> AppResult<Vec<Class>> {
        db.with_conn(|conn| SqliteClassRepository::new(conn).list())
    }

    pub fn create_class(db: &Db, input: CreateClassInput) -> AppResult<Class> {
        if input.name.trim().is_empty() {
            return Err(AppError::validation("Class name is required"));
        }
        if input.grade_level < 0 {
            return Err(AppError::validation("Grade level cannot be negative"));
        }
        if input.capacity <= 0 {
            return Err(AppError::validation("Capacity must be greater than zero"));
        }

        let class = Class {
            id: Uuid::new_v4().to_string(),
            name: input.name.trim().to_string(),
            grade_level: input.grade_level,
            capacity: input.capacity,
        };

        db.execute(|conn| SqliteClassRepository::new(conn).create(&class))?;
        Ok(class)
    }

    pub fn list_terms(db: &Db) -> AppResult<Vec<Term>> {
        db.with_conn(|conn| SqliteTermRepository::new(conn).list())
    }

    pub fn get_active_term(db: &Db) -> AppResult<Option<Term>> {
        db.with_conn(|conn| SqliteTermRepository::new(conn).get_active())
    }

    /// Creates a term. Only one term can be active at a time, so activating a
    /// term deactivates the previous one inside the same transaction.
    pub fn create_term(db: &Db, input: CreateTermInput) -> AppResult<Term> {
        if !(1..=4).contains(&input.term_number) {
            return Err(AppError::validation("Term number must be between 1 and 4"));
        }
        if input.start_date.trim().is_empty() || input.end_date.trim().is_empty() {
            return Err(AppError::validation("Start and end dates are required"));
        }
        if input.end_date < input.start_date {
            return Err(AppError::validation("End date cannot precede the start date"));
        }

        let term = Term {
            id: Uuid::new_v4().to_string(),
            year: input.year,
            term_number: input.term_number,
            start_date: input.start_date.trim().to_string(),
            end_date: input.end_date.trim().to_string(),
            is_active: input.is_active,
        };

        db.execute(|conn| {
            if term.is_active {
                conn.execute("UPDATE terms SET is_active = 0 WHERE is_active = 1", [])?;
            }
            SqliteTermRepository::new(conn).create(&term)
        })?;

        Ok(term)
    }

    /// Enrolls a student into a class for a term.
    pub fn enroll(db: &Db, input: EnrollStudentInput) -> AppResult<Enrollment> {
        if input.student_id.trim().is_empty()
            || input.class_id.trim().is_empty()
            || input.term_id.trim().is_empty()
        {
            return Err(AppError::validation(
                "Student, class and term are all required",
            ));
        }

        let enrollment = Enrollment {
            id: Uuid::new_v4().to_string(),
            student_id: input.student_id.trim().to_string(),
            class_id: input.class_id.trim().to_string(),
            term_id: input.term_id.trim().to_string(),
            status: "ACTIVE".to_string(),
            enrolled_at: crate::application::now_iso(),
        };

        db.execute(|conn| {
            let enrollments = SqliteEnrollmentRepository::new(conn);

            if enrollments
                .find_by_student_term(&enrollment.student_id, &enrollment.term_id)?
                .is_some()
            {
                return Err(AppError::conflict(
                    "The student is already enrolled in this term",
                ));
            }

            let class = SqliteClassRepository::new(conn)
                .find_by_id(&enrollment.class_id)?
                .ok_or_else(|| AppError::not_found("Class", &enrollment.class_id))?;

            let enrolled = enrollments.count_by_class_term(&enrollment.class_id, &enrollment.term_id)?;
            if enrolled >= class.capacity {
                return Err(AppError::conflict(format!(
                    "Class {} is full ({} students)",
                    class.name, class.capacity
                )));
            }

            enrollments.create(&enrollment)
        })?;

        Ok(enrollment)
    }

    pub fn list_by_class_term(
        db: &Db,
        class_id: &str,
        term_id: &str,
    ) -> AppResult<Vec<Enrollment>> {
        db.with_conn(|conn| {
            SqliteEnrollmentRepository::new(conn).list_by_class_term(class_id, term_id)
        })
    }

    pub fn list_by_student(db: &Db, student_id: &str) -> AppResult<Vec<Enrollment>> {
        db.with_conn(|conn| SqliteEnrollmentRepository::new(conn).list_by_student(student_id))
    }
}
