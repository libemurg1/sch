use crate::domain::dto::{
    CreateClassInput, CreateTermInput, EnrollStudentInput, UpdateClassInput, UpdateTermInput,
};
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

    pub fn update_class(db: &Db, input: UpdateClassInput) -> AppResult<Class> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(AppError::validation("Class name is required"));
        }
        if input.grade_level < 0 {
            return Err(AppError::validation("Grade level cannot be negative"));
        }
        if input.capacity <= 0 {
            return Err(AppError::validation("Capacity must be greater than zero"));
        }

        db.transaction(|conn| {
            let repo = SqliteClassRepository::new(conn);
            repo.find_by_id(&input.id)?
                .ok_or_else(|| AppError::not_found("Class", &input.id))?;

            if let Some(other) = repo.find_by_name(&name)? {
                if other.id != input.id {
                    return Err(AppError::conflict(format!(
                        "A class named \"{}\" already exists",
                        name
                    )));
                }
            }

            let class = Class {
                id: input.id.clone(),
                name: name.clone(),
                grade_level: input.grade_level,
                capacity: input.capacity,
            };
            repo.update(&class)?;
            Ok(class)
        })
    }

    pub fn delete_class(db: &Db, class_id: &str) -> AppResult<()> {
        db.transaction(|conn| {
            let repo = SqliteClassRepository::new(conn);
            let class = repo
                .find_by_id(class_id)?
                .ok_or_else(|| AppError::not_found("Class", class_id))?;

            if repo.reference_count(class_id)? > 0 {
                return Err(AppError::conflict(format!(
                    "\"{}\" is used by enrollments, fee structures or invoices and cannot be deleted. Rename it instead.",
                    class.name
                )));
            }

            repo.delete(class_id)
        })
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
            let repo = SqliteTermRepository::new(conn);
            if repo.find_by_year_term(term.year, term.term_number)?.is_some() {
                return Err(AppError::conflict(format!(
                    "Term {} of {} already exists",
                    term.term_number, term.year
                )));
            }
            if term.is_active {
                conn.execute("UPDATE terms SET is_active = 0 WHERE is_active = 1", [])?;
            }
            repo.create(&term)
        })?;

        Ok(term)
    }

    pub fn update_term(db: &Db, input: UpdateTermInput) -> AppResult<Term> {
        if !(1..=4).contains(&input.term_number) {
            return Err(AppError::validation("Term number must be between 1 and 4"));
        }
        if input.start_date.trim().is_empty() || input.end_date.trim().is_empty() {
            return Err(AppError::validation("Start and end dates are required"));
        }
        if input.end_date < input.start_date {
            return Err(AppError::validation("End date cannot precede the start date"));
        }

        db.transaction(|conn| {
            let repo = SqliteTermRepository::new(conn);
            repo.find_by_id(&input.id)?
                .ok_or_else(|| AppError::not_found("Term", &input.id))?;

            if let Some(other) = repo.find_by_year_term(input.year, input.term_number)? {
                if other.id != input.id {
                    return Err(AppError::conflict(format!(
                        "Term {} of {} already exists",
                        input.term_number, input.year
                    )));
                }
            }

            let term = Term {
                id: input.id.clone(),
                year: input.year,
                term_number: input.term_number,
                start_date: input.start_date.trim().to_string(),
                end_date: input.end_date.trim().to_string(),
                is_active: input.is_active,
            };

            if term.is_active {
                repo.deactivate_others(&term.id)?;
            }
            repo.update(&term)?;
            Ok(term)
        })
    }

    pub fn delete_term(db: &Db, term_id: &str) -> AppResult<()> {
        db.transaction(|conn| {
            let repo = SqliteTermRepository::new(conn);
            let term = repo
                .find_by_id(term_id)?
                .ok_or_else(|| AppError::not_found("Term", term_id))?;

            if repo.reference_count(term_id)? > 0 {
                return Err(AppError::conflict(format!(
                    "Term {} of {} is used by enrollments, fees, invoices or discounts and cannot be deleted.",
                    term.term_number, term.year
                )));
            }

            repo.delete(term_id)
        })
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (Db, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "sfs-enrollment-{}-{}",
            name,
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let db = Db::open(&dir).unwrap();
        (db, dir)
    }

    fn seed_student(db: &Db) {
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO students (id, student_number, first_name, last_name, dob, admission_date) \
                 VALUES ('s1', 'STU-1', 'Test', 'Student', '2015-01-01', '2026-01-01')",
                [],
            )?;
            Ok(())
        })
        .unwrap();
    }

    fn make_class(db: &Db, name: &str) -> Class {
        EnrollmentService::create_class(
            db,
            CreateClassInput {
                name: name.into(),
                grade_level: 1,
                capacity: 20,
            },
        )
        .unwrap()
    }

    fn make_term(db: &Db) -> Term {
        EnrollmentService::create_term(
            db,
            CreateTermInput {
                year: 2026,
                term_number: 1,
                start_date: "2026-01-01".into(),
                end_date: "2026-04-01".into(),
                is_active: true,
            },
        )
        .unwrap()
    }

    #[test]
    fn classes_and_terms_can_be_edited_and_deleted_when_unused() {
        let (db, dir) = temp_db("crud");
        let class = make_class(&db, "Grade 1 Blue");
        let term = make_term(&db);

        let renamed = EnrollmentService::update_class(
            &db,
            UpdateClassInput {
                id: class.id.clone(),
                name: "Grade 1 Green".into(),
                grade_level: 1,
                capacity: 25,
            },
        )
        .unwrap();
        assert_eq!(renamed.name, "Grade 1 Green");
        assert_eq!(renamed.capacity, 25);

        EnrollmentService::delete_class(&db, &class.id).unwrap();
        assert!(EnrollmentService::list_classes(&db).unwrap().is_empty());

        let edited = EnrollmentService::update_term(
            &db,
            UpdateTermInput {
                id: term.id.clone(),
                year: 2026,
                term_number: 1,
                start_date: "2026-01-02".into(),
                end_date: "2026-04-02".into(),
                is_active: true,
            },
        )
        .unwrap();
        assert_eq!(edited.start_date, "2026-01-02");

        EnrollmentService::delete_term(&db, &term.id).unwrap();
        assert!(EnrollmentService::list_terms(&db).unwrap().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_class_or_term_in_use_cannot_be_deleted() {
        let (db, dir) = temp_db("in-use");
        seed_student(&db);
        let class = make_class(&db, "Grade 1 Blue");
        let term = make_term(&db);

        EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: class.id.clone(),
                term_id: term.id.clone(),
            },
        )
        .unwrap();

        assert_eq!(
            EnrollmentService::delete_class(&db, &class.id).unwrap_err().code,
            "CONFLICT"
        );
        assert_eq!(
            EnrollmentService::delete_term(&db, &term.id).unwrap_err().code,
            "CONFLICT"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
