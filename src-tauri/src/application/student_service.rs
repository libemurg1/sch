use crate::application::{current_year, now_iso};
use crate::domain::dto::{RegisterStudentInput, UpdateStudentInput};
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
        validate_registration(&input)?;
        db.execute(|conn| register_with_conn(conn, input))
    }

    /// Registers a CSV batch atomically so validation or storage failures do
    /// not leave the school with a partially imported roster.
    pub fn register_many(db: &Db, inputs: Vec<RegisterStudentInput>) -> AppResult<Vec<Student>> {
        if inputs.is_empty() {
            return Err(AppError::validation(
                "Choose a CSV file containing at least one student",
            ));
        }
        for input in &inputs {
            validate_registration(input)?;
        }

        db.execute(|conn| {
            inputs
                .into_iter()
                .map(|input| register_with_conn(conn, input))
                .collect()
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

    /// Changes the editable details of an existing student. The student
    /// number, status and creation time are immutable.
    pub fn update_details(db: &Db, input: UpdateStudentInput) -> AppResult<Student> {
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
            let mut student = students
                .find_by_id(input.student_id.trim())?
                .ok_or_else(|| AppError::not_found("Student", &input.student_id))?;

            student.first_name = input.first_name.trim().to_string();
            student.last_name = input.last_name.trim().to_string();
            student.dob = input.dob.trim().to_string();
            student.gender = normalize_optional(input.gender.as_deref());
            student.admission_date = input.admission_date.trim().to_string();
            student.guardian_name = normalize_optional(input.guardian_name.as_deref());
            student.guardian_phone = normalize_optional(input.guardian_phone.as_deref());
            student.guardian_email = normalize_optional(input.guardian_email.as_deref());

            students.update_details(&student)?;
            Ok(student)
        })
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

fn validate_registration(input: &RegisterStudentInput) -> AppResult<()> {
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
    Ok(())
}

fn register_with_conn(
    conn: &rusqlite::Connection,
    input: RegisterStudentInput,
) -> AppResult<Student> {
    let students = SqliteStudentRepository::new(conn);
    let numbers = NumberGenerator::new(conn);
    let student_number = numbers.next_formatted("STU", current_year(), 4)?;
    let student = Student {
        id: Uuid::new_v4().to_string(),
        student_number,
        first_name: input.first_name.trim().to_string(),
        last_name: input.last_name.trim().to_string(),
        dob: input.dob.trim().to_string(),
        gender: normalize_optional(input.gender.as_deref()),
        admission_date: input.admission_date.trim().to_string(),
        guardian_name: normalize_optional(input.guardian_name.as_deref()),
        guardian_phone: normalize_optional(input.guardian_phone.as_deref()),
        guardian_email: normalize_optional(input.guardian_email.as_deref()),
        status: "ACTIVE".to_string(),
        created_at: now_iso(),
    };
    students.create(&student)?;
    Ok(student)
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (Db, PathBuf) {
        let dir = std::env::temp_dir().join(format!("sfs-student-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let db = Db::open(&dir).unwrap();
        (db, dir)
    }

    fn register(db: &Db) -> Student {
        StudentService::register(
            db,
            RegisterStudentInput {
                first_name: "Ada".into(),
                last_name: "Lovelace".into(),
                dob: "2015-01-01".into(),
                gender: Some("Female".into()),
                admission_date: "2026-01-01".into(),
                guardian_name: Some("Grace".into()),
                guardian_phone: Some("123".into()),
                guardian_email: Some("grace@example.com".into()),
            },
        )
        .unwrap()
    }

    #[test]
    fn bulk_registration_creates_all_students_and_rejects_invalid_batches() {
        let (db, dir) = temp_db("bulk-registration");
        let first = RegisterStudentInput {
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            dob: "2015-01-01".into(),
            gender: None,
            admission_date: "2026-01-01".into(),
            guardian_name: None,
            guardian_phone: None,
            guardian_email: None,
        };
        let second = RegisterStudentInput {
            first_name: "Grace".into(),
            last_name: "Hopper".into(),
            ..first.clone()
        };

        let imported = StudentService::register_many(&db, vec![first.clone(), second]).unwrap();
        assert_eq!(imported.len(), 2);
        assert_ne!(imported[0].student_number, imported[1].student_number);

        let mut invalid = first.clone();
        invalid.first_name = " ".into();
        assert_eq!(
            StudentService::register_many(&db, vec![first.clone(), invalid])
                .unwrap_err()
                .code,
            "VALIDATION"
        );
        assert_eq!(StudentService::list(&db).unwrap().len(), 2);

        let _ = fs::remove_dir_all(&dir);
    }

    fn update_input(student_id: &str) -> UpdateStudentInput {
        UpdateStudentInput {
            student_id: student_id.to_string(),
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            dob: "2015-01-01".into(),
            gender: Some("Female".into()),
            admission_date: "2026-01-01".into(),
            guardian_name: Some("Grace".into()),
            guardian_phone: Some("123".into()),
            guardian_email: Some("grace@example.com".into()),
        }
    }

    #[test]
    fn update_changes_editable_details_but_not_identity() {
        let (db, dir) = temp_db("update");
        let created = register(&db);

        let updated = StudentService::update_details(
            &db,
            UpdateStudentInput {
                student_id: created.id.clone(),
                first_name: "  Ada  ".into(),
                last_name: "  Byron  ".into(),
                dob: " 2014-12-31 ".into(),
                gender: Some("  ".into()),
                admission_date: " 2025-09-01 ".into(),
                guardian_name: Some("  Anne  ".into()),
                guardian_phone: Some("   ".into()),
                guardian_email: Some("anne@example.com".into()),
            },
        )
        .unwrap();

        assert_eq!(updated.first_name, "Ada");
        assert_eq!(updated.last_name, "Byron");
        assert_eq!(updated.dob, "2014-12-31");
        assert_eq!(updated.gender, None);
        assert_eq!(updated.admission_date, "2025-09-01");
        assert_eq!(updated.guardian_name.as_deref(), Some("Anne"));
        assert_eq!(updated.guardian_phone, None);
        assert_eq!(updated.guardian_email.as_deref(), Some("anne@example.com"));
        assert_eq!(updated.student_number, created.student_number);
        assert_eq!(updated.status, created.status);
        assert_eq!(updated.created_at, created.created_at);

        let stored = StudentService::find(&db, &created.id).unwrap().unwrap();
        assert_eq!(stored.first_name, "Ada");
        assert_eq!(stored.last_name, "Byron");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn update_validates_required_fields_and_missing_students() {
        let (db, dir) = temp_db("update-validation");
        let created = register(&db);

        let mut input = update_input(&created.id);
        input.first_name = "   ".into();
        assert_eq!(
            StudentService::update_details(&db, input.clone())
                .unwrap_err()
                .code,
            "VALIDATION"
        );

        input = update_input(&created.id);
        input.last_name = "".into();
        assert_eq!(
            StudentService::update_details(&db, input.clone())
                .unwrap_err()
                .code,
            "VALIDATION"
        );

        input = update_input(&created.id);
        input.dob = "".into();
        assert_eq!(
            StudentService::update_details(&db, input.clone())
                .unwrap_err()
                .code,
            "VALIDATION"
        );

        input = update_input(&created.id);
        input.admission_date = "".into();
        assert_eq!(
            StudentService::update_details(&db, input.clone())
                .unwrap_err()
                .code,
            "VALIDATION"
        );

        input = update_input("missing");
        assert_eq!(
            StudentService::update_details(&db, input).unwrap_err().code,
            "NOT_FOUND"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
