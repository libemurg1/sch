use crate::domain::dto::{
    CreateClassInput, CreateTermInput, EnrollStudentInput, MoveStudentInput, UpdateClassInput,
    UpdateTermInput,
};
use crate::domain::entities::{Class, Enrollment, Grade, Term};
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::{ClassRepository, EnrollmentRepository, TermRepository};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::class_repository::SqliteClassRepository;
use crate::infrastructure::sqlite::enrollment_repository::SqliteEnrollmentRepository;
use crate::infrastructure::sqlite::term_repository::SqliteTermRepository;
use crate::infrastructure::sqlite::unit_of_work::UnitOfWork;
use rusqlite::Connection;
use uuid::Uuid;

/// Use cases for classes, terms and enrollments.
pub struct EnrollmentService;

impl EnrollmentService {
    pub fn list_classes(db: &Db, year: Option<i32>) -> AppResult<Vec<Class>> {
        db.with_conn(|conn| match year {
            Some(year) => SqliteClassRepository::new(conn).list_by_year(year),
            None => SqliteClassRepository::new(conn).list(),
        })
    }

    pub fn list_grades(db: &Db) -> AppResult<Vec<Grade>> {
        db.with_conn(|conn| SqliteClassRepository::new(conn).list_grades())
    }

    pub fn create_class(db: &Db, input: CreateClassInput) -> AppResult<Class> {
        validate_class_input(&input)?;

        db.transaction(|conn| {
            let repo = SqliteClassRepository::new(conn);
            let name = compose_class_name(conn, input.grade_level, &input.stream)?;

            if repo
                .find_by_year_name(input.academic_year, &name)?
                .is_some()
            {
                return Err(AppError::conflict(format!(
                    "\"{}\" already exists for {}",
                    name, input.academic_year
                )));
            }

            let class = Class {
                id: Uuid::new_v4().to_string(),
                name,
                grade_level: input.grade_level,
                stream: input.stream.trim().to_string(),
                academic_year: input.academic_year,
                capacity: input.capacity,
            };
            repo.create(&class)?;
            Ok(class)
        })
    }

    pub fn update_class(db: &Db, input: UpdateClassInput) -> AppResult<Class> {
        validate_update_class_input(&input)?;

        db.transaction(|conn| {
            let repo = SqliteClassRepository::new(conn);
            let existing = repo
                .find_by_id(&input.id)?
                .ok_or_else(|| AppError::not_found("Class", &input.id))?;

            if input.academic_year != existing.academic_year
                && repo.reference_count(&input.id)? > 0
            {
                return Err(AppError::conflict(
                    "This class already has students, fees or promotions. Create a class in the new year instead of moving it.",
                ));
            }

            let name = compose_class_name(conn, input.grade_level, &input.stream)?;
            if let Some(other) = repo.find_by_year_name(input.academic_year, &name)? {
                if other.id != input.id {
                    return Err(AppError::conflict(format!(
                        "\"{}\" already exists for {}",
                        name, input.academic_year
                    )));
                }
            }

            let class = Class {
                id: input.id.clone(),
                name,
                grade_level: input.grade_level,
                stream: input.stream.trim().to_string(),
                academic_year: input.academic_year,
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
                    "\"{}\" is used by students, fees or promotions and cannot be deleted. Rename it instead.",
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
            return Err(AppError::validation(
                "End date cannot precede the start date",
            ));
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
            if repo
                .find_by_year_term(term.year, term.term_number)?
                .is_some()
            {
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
            return Err(AppError::validation(
                "End date cannot precede the start date",
            ));
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
                    "Term {} of {} is used by students, fees, invoices or discounts and cannot be deleted.",
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
            ended_at: None,
            end_reason: None,
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

            let term = SqliteTermRepository::new(conn)
                .find_by_id(&enrollment.term_id)?
                .ok_or_else(|| AppError::not_found("Term", &enrollment.term_id))?;

            if class.academic_year != term.year {
                return Err(AppError::validation(format!(
                    "{} belongs to {} but the term is in {}",
                    class.name, class.academic_year, term.year
                )));
            }

            let enrolled =
                enrollments.count_by_class_term(&enrollment.class_id, &enrollment.term_id)?;
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

    /// Moves a student to another class (stream) within the same term.
    pub fn move_student(db: &Db, input: MoveStudentInput) -> AppResult<Enrollment> {
        if input.student_id.trim().is_empty()
            || input.class_id.trim().is_empty()
            || input.term_id.trim().is_empty()
        {
            return Err(AppError::validation(
                "Student, class and term are all required",
            ));
        }

        db.transaction(|conn| {
            let enrollments = SqliteEnrollmentRepository::new(conn);
            let enrollment = enrollments
                .find_by_student_term(&input.student_id, &input.term_id)?
                .ok_or_else(|| {
                    AppError::not_found("Enrollment", &format!("{} in this term", input.student_id))
                })?;

            if enrollment.class_id == input.class_id {
                return Ok(enrollment);
            }

            let class = SqliteClassRepository::new(conn)
                .find_by_id(&input.class_id)?
                .ok_or_else(|| AppError::not_found("Class", &input.class_id))?;

            let term = SqliteTermRepository::new(conn)
                .find_by_id(&input.term_id)?
                .ok_or_else(|| AppError::not_found("Term", &input.term_id))?;

            if class.academic_year != term.year {
                return Err(AppError::validation(format!(
                    "{} belongs to {} but the term is in {}",
                    class.name, class.academic_year, term.year
                )));
            }

            let enrolled = enrollments.count_by_class_term(&input.class_id, &input.term_id)?;
            if enrolled >= class.capacity {
                return Err(AppError::conflict(format!(
                    "Class {} is full ({} students)",
                    class.name, class.capacity
                )));
            }

            enrollments.update_class(&enrollment.id, &input.class_id)?;
            Ok(Enrollment {
                class_id: input.class_id.clone(),
                ..enrollment
            })
        })
    }

    /// The class roster for a term, including historical inactive enrollments.
    pub fn list_by_class_term(
        db: &Db,
        class_id: &str,
        term_id: &str,
    ) -> AppResult<Vec<Enrollment>> {
        db.with_conn(|conn| {
            SqliteEnrollmentRepository::new(conn).list_by_class_term_all_statuses(class_id, term_id)
        })
    }

    /// Every enrollment in a term, including inactive historical rows.
    pub fn list_by_term(db: &Db, term_id: &str) -> AppResult<Vec<Enrollment>> {
        db.with_conn(|conn| SqliteEnrollmentRepository::new(conn).list_by_term(term_id))
    }

    pub fn list_by_student(db: &Db, student_id: &str) -> AppResult<Vec<Enrollment>> {
        db.with_conn(|conn| SqliteEnrollmentRepository::new(conn).list_by_student(student_id))
    }
}

fn validate_class_input(input: &CreateClassInput) -> AppResult<()> {
    if !(2000..=2100).contains(&input.academic_year) {
        return Err(AppError::validation("Choose a valid academic year"));
    }
    if input.capacity <= 0 {
        return Err(AppError::validation("Capacity must be greater than zero"));
    }
    Ok(())
}

fn validate_update_class_input(input: &UpdateClassInput) -> AppResult<()> {
    if !(2000..=2100).contains(&input.academic_year) {
        return Err(AppError::validation("Choose a valid academic year"));
    }
    if input.capacity <= 0 {
        return Err(AppError::validation("Capacity must be greater than zero"));
    }
    Ok(())
}

/// Builds the display name ("Grade 1 Blue") from the grade and stream.
fn compose_class_name(conn: &Connection, grade_level: i32, stream: &str) -> AppResult<String> {
    let grade = SqliteClassRepository::new(conn)
        .find_grade(grade_level)?
        .ok_or_else(|| AppError::validation("Choose a grade"))?;

    let stream = stream.trim();
    if stream.chars().count() > 40 {
        return Err(AppError::validation(
            "The stream must be 40 characters or fewer",
        ));
    }

    Ok(if stream.is_empty() {
        grade.name
    } else {
        format!("{} {}", grade.name, stream)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (Db, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("sfs-enrollment-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let db = Db::open(&dir).unwrap();
        // Migrations seed the standard classes; tests build their own world.
        db.transaction(|conn| {
            conn.execute("DELETE FROM classes", [])?;
            conn.execute("DELETE FROM terms", [])?;
            Ok(())
        })
        .unwrap();
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

    fn make_class(db: &Db, stream: &str) -> Class {
        EnrollmentService::create_class(
            db,
            CreateClassInput {
                grade_level: 1,
                stream: stream.into(),
                academic_year: 2026,
                capacity: 20,
            },
        )
        .unwrap()
    }

    fn make_term(db: &Db, year: i32) -> Term {
        EnrollmentService::create_term(
            db,
            CreateTermInput {
                year,
                term_number: 1,
                start_date: format!("{year}-01-01"),
                end_date: format!("{year}-04-01"),
                is_active: true,
            },
        )
        .unwrap()
    }

    #[test]
    fn class_names_come_from_the_grade_and_stream() {
        let (db, dir) = temp_db("names");
        let class = make_class(&db, "Blue");
        assert_eq!(class.name, "Grade 1 Blue");
        assert_eq!(class.stream, "Blue");
        assert_eq!(class.academic_year, 2026);

        assert_eq!(
            EnrollmentService::create_class(
                &db,
                CreateClassInput {
                    grade_level: 1,
                    stream: "Blue".into(),
                    academic_year: 2026,
                    capacity: 20,
                },
            )
            .unwrap_err()
            .code,
            "CONFLICT"
        );

        // The same class name is allowed in a different year.
        EnrollmentService::create_class(
            &db,
            CreateClassInput {
                grade_level: 1,
                stream: "Blue".into(),
                academic_year: 2027,
                capacity: 20,
            },
        )
        .unwrap();

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn classes_and_terms_can_be_edited_and_deleted_when_unused() {
        let (db, dir) = temp_db("crud");
        let class = make_class(&db, "Blue");
        let term = make_term(&db, 2026);

        let renamed = EnrollmentService::update_class(
            &db,
            UpdateClassInput {
                id: class.id.clone(),
                grade_level: 1,
                stream: "Green".into(),
                academic_year: 2026,
                capacity: 25,
            },
        )
        .unwrap();
        assert_eq!(renamed.name, "Grade 1 Green");
        assert_eq!(renamed.capacity, 25);

        EnrollmentService::delete_class(&db, &class.id).unwrap();
        assert!(EnrollmentService::list_classes(&db, None)
            .unwrap()
            .is_empty());

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
        let class = make_class(&db, "Blue");
        let term = make_term(&db, 2026);

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
            EnrollmentService::delete_class(&db, &class.id)
                .unwrap_err()
                .code,
            "CONFLICT"
        );
        assert_eq!(
            EnrollmentService::delete_term(&db, &term.id)
                .unwrap_err()
                .code,
            "CONFLICT"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_student_can_move_to_another_stream_in_the_same_term() {
        let (db, dir) = temp_db("move");
        seed_student(&db);
        let blue = make_class(&db, "Blue");
        let yellow = make_class(&db, "Yellow");
        let term = make_term(&db, 2026);

        EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: blue.id.clone(),
                term_id: term.id.clone(),
            },
        )
        .unwrap();

        let moved = EnrollmentService::move_student(
            &db,
            MoveStudentInput {
                student_id: "s1".into(),
                term_id: term.id.clone(),
                class_id: yellow.id.clone(),
            },
        )
        .unwrap();
        assert_eq!(moved.class_id, yellow.id);

        let roster = EnrollmentService::list_by_class_term(&db, &yellow.id, &term.id).unwrap();
        assert_eq!(roster.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn historical_roster_includes_inactive_enrollments_but_active_list_does_not() {
        let (db, dir) = temp_db("roster-history");
        seed_student(&db);
        let class = make_class(&db, "Blue");
        let term = make_term(&db, 2026);

        EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: class.id.clone(),
                term_id: term.id.clone(),
            },
        )
        .unwrap();

        db.transaction(|conn| {
            conn.execute("UPDATE enrollments SET status = 'INACTIVE'", [])?;
            Ok(())
        })
        .unwrap();

        let roster = EnrollmentService::list_by_class_term(&db, &class.id, &term.id).unwrap();
        assert_eq!(roster.len(), 1);
        assert_eq!(roster[0].status, "INACTIVE");

        let active = db
            .with_conn(|conn| {
                SqliteEnrollmentRepository::new(conn).list_by_class_term(&class.id, &term.id)
            })
            .unwrap();
        assert!(active.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn term_roster_includes_inactive_rows_and_excludes_other_terms() {
        let (db, dir) = temp_db("roster-term");
        seed_student(&db);
        let class_2026 = make_class(&db, "Blue");
        let term_2026 = make_term(&db, 2026);
        let class_2027 = EnrollmentService::create_class(
            &db,
            CreateClassInput {
                grade_level: 1,
                stream: "Blue".into(),
                academic_year: 2027,
                capacity: 20,
            },
        )
        .unwrap();
        let term_2027 = make_term(&db, 2027);

        EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: class_2026.id.clone(),
                term_id: term_2026.id.clone(),
            },
        )
        .unwrap();
        EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: class_2027.id,
                term_id: term_2027.id.clone(),
            },
        )
        .unwrap();

        db.transaction(|conn| {
            conn.execute(
                "UPDATE enrollments SET status = 'INACTIVE' WHERE term_id = ?",
                rusqlite::params![term_2026.id],
            )?;
            Ok(())
        })
        .unwrap();

        let roster_2026 = EnrollmentService::list_by_term(&db, &term_2026.id).unwrap();
        assert_eq!(roster_2026.len(), 1);
        assert_eq!(roster_2026[0].status, "INACTIVE");

        let roster_2027 = EnrollmentService::list_by_term(&db, &term_2027.id).unwrap();
        assert_eq!(roster_2027.len(), 1);
        assert_eq!(roster_2027[0].status, "ACTIVE");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_class_from_another_year_cannot_be_used_for_a_term() {
        let (db, dir) = temp_db("wrong-year");
        seed_student(&db);
        let class = make_class(&db, "Blue");
        let term = make_term(&db, 2027);

        let error = EnrollmentService::enroll(
            &db,
            EnrollStudentInput {
                student_id: "s1".into(),
                class_id: class.id,
                term_id: term.id,
            },
        )
        .unwrap_err();
        assert_eq!(error.code, "VALIDATION");

        let _ = fs::remove_dir_all(&dir);
    }
}
