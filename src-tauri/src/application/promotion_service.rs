use crate::domain::dto::{
    ApplyPromotionInput, PromotionCandidate, PromotionPreview, PromotionResult,
};
use crate::domain::entities::Class;
use crate::domain::errors::{AppError, AppResult};
use crate::domain::ports::{ClassRepository, EnrollmentRepository, TermRepository};
use crate::infrastructure::db::Db;
use crate::infrastructure::sqlite::class_repository::SqliteClassRepository;
use crate::infrastructure::sqlite::enrollment_repository::SqliteEnrollmentRepository;
use crate::infrastructure::sqlite::term_repository::SqliteTermRepository;
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

/// End-of-year movement: copy the class structure into the next year, preview
/// who is ready to move, then apply promotions, repeats and graduations.
pub struct PromotionService;

struct CandidateRow {
    student_id: String,
    student_number: String,
    first_name: String,
    last_name: String,
    class_id: String,
    class_name: String,
    grade_level: i32,
    stream: String,
}

impl PromotionService {
    pub fn preview(db: &Db, from_year: i32, to_year: i32) -> AppResult<PromotionPreview> {
        validate_years(from_year, to_year)?;

        db.with_conn(|conn| {
            let term_ready = SqliteTermRepository::new(conn)
                .find_by_year_term(to_year, 1)?
                .is_some();

            let mut stmt = conn.prepare(
                "SELECT s.id, s.student_number, s.first_name, s.last_name, \
                        c.id, c.name, c.grade_level, c.stream \
                 FROM enrollments e \
                 JOIN terms t ON t.id = e.term_id \
                 JOIN students s ON s.id = e.student_id \
                 JOIN classes c ON c.id = e.class_id \
                 WHERE t.year = ?1 \
                   AND e.status = 'ACTIVE' \
                   AND s.status = 'ACTIVE' \
                   AND t.term_number = ( \
                     SELECT MAX(t2.term_number) \
                     FROM enrollments e2 \
                     JOIN terms t2 ON t2.id = e2.term_id \
                     WHERE e2.student_id = e.student_id \
                       AND t2.year = t.year \
                       AND e2.status = 'ACTIVE' \
                   ) \
                   AND NOT EXISTS ( \
                     SELECT 1 FROM promotions p \
                     WHERE p.student_id = s.id AND p.to_year = ?2 \
                   ) \
                   AND NOT EXISTS ( \
                     SELECT 1 FROM enrollments e3 \
                     JOIN terms t3 ON t3.id = e3.term_id \
                     WHERE e3.student_id = s.id AND t3.year = ?2 \
                   ) \
                 ORDER BY c.grade_level, c.stream, s.last_name, s.first_name",
            )?;

            let rows = stmt.query_map(params![from_year, to_year], |row| {
                Ok(CandidateRow {
                    student_id: row.get(0)?,
                    student_number: row.get(1)?,
                    first_name: row.get(2)?,
                    last_name: row.get(3)?,
                    class_id: row.get(4)?,
                    class_name: row.get(5)?,
                    grade_level: row.get(6)?,
                    stream: row.get(7)?,
                })
            })?;

            let mut candidates = Vec::new();
            for row in rows {
                let row = row?;
                let (suggested_outcome, suggested_class_id) =
                    suggest_target(conn, to_year, row.grade_level, &row.stream)?;
                candidates.push(PromotionCandidate {
                    student_id: row.student_id,
                    student_number: row.student_number,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    from_class_id: row.class_id,
                    from_class_name: row.class_name,
                    from_grade_level: row.grade_level,
                    from_stream: row.stream,
                    suggested_outcome,
                    suggested_class_id,
                });
            }

            Ok(PromotionPreview {
                from_year,
                to_year,
                term_ready,
                candidates,
            })
        })
    }

    pub fn apply(
        db: &Db,
        input: ApplyPromotionInput,
        actor_id: Option<&str>,
    ) -> AppResult<PromotionResult> {
        validate_years(input.from_year, input.to_year)?;
        if input.entries.is_empty() {
            return Ok(PromotionResult {
                promoted: 0,
                repeated: 0,
                graduated: 0,
            });
        }

        db.transaction(|conn| {
            let term = SqliteTermRepository::new(conn)
                .find_by_year_term(input.to_year, 1)?
                .ok_or_else(|| {
                    AppError::conflict(format!(
                        "Create Term 1 of {} first, then run the promotion.",
                        input.to_year
                    ))
                })?;

            let enrollments = SqliteEnrollmentRepository::new(conn);
            let classes = SqliteClassRepository::new(conn);
            let mut result = PromotionResult {
                promoted: 0,
                repeated: 0,
                graduated: 0,
            };
            let mut seen = std::collections::HashSet::new();

            for entry in &input.entries {
                if !seen.insert(entry.student_id.clone()) {
                    return Err(AppError::validation(
                        "The same student appears more than once",
                    ));
                }

                let source =
                    source_class(conn, &entry.student_id, input.from_year)?.ok_or_else(|| {
                        AppError::conflict(format!(
                            "The student was not enrolled in {}",
                            input.from_year
                        ))
                    })?;

                let outcome = entry.outcome.trim().to_uppercase();
                let mut to_class: Option<Class> = None;

                match outcome.as_str() {
                    "GRADUATED" => {
                        if source.grade_level < 7 {
                            return Err(AppError::validation(format!(
                                "Only Grade 7 students can graduate ({} is in {})",
                                entry.student_id,
                                grade_name(conn, source.grade_level)?
                            )));
                        }
                    }
                    "PROMOTED" | "REPEATED" => {
                        let class_id = entry.to_class_id.as_deref().ok_or_else(|| {
                            AppError::validation("Choose the class to move the student into")
                        })?;
                        let class = classes
                            .find_by_id(class_id)?
                            .ok_or_else(|| AppError::not_found("Class", class_id))?;

                        if class.academic_year != input.to_year {
                            return Err(AppError::validation(format!(
                                "{} belongs to {}",
                                class.name, class.academic_year
                            )));
                        }

                        if outcome == "PROMOTED" && class.grade_level != source.grade_level + 1 {
                            return Err(AppError::validation(format!(
                                "{} cannot be promoted into {}",
                                source.class_name, class.name
                            )));
                        }
                        if outcome == "REPEATED" && class.grade_level != source.grade_level {
                            return Err(AppError::validation(format!(
                                "{} can only repeat {}",
                                source.class_name,
                                grade_name(conn, source.grade_level)?
                            )));
                        }

                        let enrolled = enrollments.count_by_class_term(&class.id, &term.id)?;
                        if enrolled >= class.capacity {
                            return Err(AppError::conflict(format!(
                                "{} is full ({} students)",
                                class.name, class.capacity
                            )));
                        }

                        to_class = Some(class);
                    }
                    _ => return Err(AppError::validation("Use PROMOTED, REPEATED or GRADUATED")),
                }

                if enrollments
                    .find_by_student_term(&entry.student_id, &term.id)?
                    .is_some()
                {
                    return Err(AppError::conflict(
                        "The student is already enrolled in the new year",
                    ));
                }

                let enrollment = crate::domain::entities::Enrollment {
                    id: Uuid::new_v4().to_string(),
                    student_id: entry.student_id.clone(),
                    class_id: to_class
                        .as_ref()
                        .map(|class| class.id.clone())
                        .unwrap_or_else(|| source.class_id.clone()),
                    term_id: term.id.clone(),
                    status: "ACTIVE".to_string(),
                    enrolled_at: crate::application::now_iso(),
                    ended_at: None,
                    end_reason: None,
                };

                // The new year becomes the student's active enrollment; every
                // older enrollment stays on record as inactive.
                enrollments.deactivate_active_for_student(&entry.student_id)?;

                if to_class.is_some() {
                    enrollments.create(&enrollment)?;
                } else {
                    conn.execute(
                        "UPDATE students SET status = 'GRADUATED' WHERE id = ?",
                        params![entry.student_id],
                    )?;
                }

                conn.execute(
                    "INSERT INTO promotions \
                       (id, student_id, from_year, to_year, from_class_id, to_class_id, \
                        outcome, created_by) \
                     VALUES (?,?,?,?,?,?,?,?)",
                    params![
                        Uuid::new_v4().to_string(),
                        entry.student_id,
                        input.from_year,
                        input.to_year,
                        source.class_id,
                        to_class.as_ref().map(|class| class.id.clone()),
                        outcome,
                        actor_id
                    ],
                )?;

                match outcome.as_str() {
                    "PROMOTED" => result.promoted += 1,
                    "REPEATED" => result.repeated += 1,
                    _ => result.graduated += 1,
                }
            }

            Ok(result)
        })
    }

    /// Copies every class of `from_year` into `to_year`, keeping grade, stream
    /// and capacity. Existing target classes are left untouched.
    pub fn copy_classes(db: &Db, from_year: i32, to_year: i32) -> AppResult<i32> {
        if to_year <= from_year {
            return Err(AppError::validation(
                "The new year must come after the current year",
            ));
        }

        db.transaction(|conn| {
            let repo = SqliteClassRepository::new(conn);
            let source = repo.list_by_year(from_year)?;
            if source.is_empty() {
                return Err(AppError::validation(format!(
                    "There are no classes in {} to copy",
                    from_year
                )));
            }

            let mut copied = 0;
            for class in source {
                if repo.find_by_year_name(to_year, &class.name)?.is_some() {
                    continue;
                }
                repo.create(&Class {
                    id: Uuid::new_v4().to_string(),
                    name: class.name,
                    grade_level: class.grade_level,
                    stream: class.stream,
                    academic_year: to_year,
                    capacity: class.capacity,
                })?;
                copied += 1;
            }
            Ok(copied)
        })
    }
}

fn validate_years(from_year: i32, to_year: i32) -> AppResult<()> {
    if to_year != from_year + 1 {
        return Err(AppError::validation(
            "Promotion moves students into the following year",
        ));
    }
    Ok(())
}

struct SourceClass {
    class_id: String,
    class_name: String,
    grade_level: i32,
}

fn source_class(conn: &Connection, student_id: &str, year: i32) -> AppResult<Option<SourceClass>> {
    Ok(conn
        .query_row(
            "SELECT c.id, c.name, c.grade_level \
             FROM enrollments e \
             JOIN terms t ON t.id = e.term_id \
             JOIN classes c ON c.id = e.class_id \
             WHERE e.student_id = ?1 AND t.year = ?2 AND e.status = 'ACTIVE' \
             ORDER BY t.term_number DESC LIMIT 1",
            params![student_id, year],
            |row| {
                Ok(SourceClass {
                    class_id: row.get(0)?,
                    class_name: row.get(1)?,
                    grade_level: row.get(2)?,
                })
            },
        )
        .optional()?)
}

fn suggest_target(
    conn: &Connection,
    to_year: i32,
    grade_level: i32,
    stream: &str,
) -> AppResult<(String, Option<String>)> {
    if grade_level >= 7 {
        return Ok(("GRADUATED".to_string(), None));
    }

    let next_grade = grade_level + 1;
    let same_stream: Option<String> = conn
        .query_row(
            "SELECT id FROM classes \
             WHERE academic_year = ?1 AND grade_level = ?2 AND stream = ?3 \
             ORDER BY name LIMIT 1",
            params![to_year, next_grade, stream],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(id) = same_stream {
        return Ok(("PROMOTED".to_string(), Some(id)));
    }

    let any_class: Option<String> = conn
        .query_row(
            "SELECT id FROM classes \
             WHERE academic_year = ?1 AND grade_level = ?2 \
             ORDER BY stream, name LIMIT 1",
            params![to_year, next_grade],
            |row| row.get(0),
        )
        .optional()?;

    Ok(("PROMOTED".to_string(), any_class))
}

fn grade_name(conn: &Connection, level: i32) -> AppResult<String> {
    Ok(conn
        .query_row(
            "SELECT name FROM grades WHERE level = ?",
            params![level],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or_else(|| format!("Grade {}", level)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::enrollment_service::EnrollmentService;
    use crate::domain::dto::{
        CreateClassInput, CreateTermInput, EnrollStudentInput, PromotionEntryInput,
    };
    use crate::domain::entities::{Class, Term};
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> (Db, PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("sfs-promotion-{}-{}", name, std::process::id()));
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

    fn seed_student(db: &Db, id: &str) {
        db.transaction(|conn| {
            conn.execute(
                "INSERT INTO students (id, student_number, first_name, last_name, dob, admission_date) \
                 VALUES (?1, ?2, 'Test', ?1, '2015-01-01', '2026-01-01')",
                params![id, format!("STU-{id}")],
            )?;
            Ok(())
        })
        .unwrap();
    }

    fn make_class(db: &Db, grade: i32, stream: &str, year: i32) -> Class {
        EnrollmentService::create_class(
            db,
            CreateClassInput {
                grade_level: grade,
                stream: stream.into(),
                academic_year: year,
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

    fn enroll(db: &Db, student: &str, class: &Class, term: &Term) {
        EnrollmentService::enroll(
            db,
            EnrollStudentInput {
                student_id: student.into(),
                class_id: class.id.clone(),
                term_id: term.id.clone(),
            },
        )
        .unwrap();
    }

    #[test]
    fn copy_classes_carries_grade_stream_and_capacity() {
        let (db, dir) = temp_db("copy");
        make_class(&db, 1, "Blue", 2026);
        make_class(&db, 4, "Green", 2026);

        let copied = PromotionService::copy_classes(&db, 2026, 2027).unwrap();
        assert_eq!(copied, 2);

        let classes = db
            .with_conn(|conn| SqliteClassRepository::new(conn).list_by_year(2027))
            .unwrap();
        assert_eq!(classes.len(), 2);
        assert!(classes
            .iter()
            .any(|class| class.name == "Grade 1 Blue" && class.stream == "Blue"));
        assert!(classes
            .iter()
            .any(|class| class.name == "Grade 4 Green" && class.capacity == 20));

        // Running again is safe and copies nothing.
        assert_eq!(PromotionService::copy_classes(&db, 2026, 2027).unwrap(), 0);
        assert_eq!(
            PromotionService::copy_classes(&db, 2027, 2027)
                .unwrap_err()
                .code,
            "VALIDATION"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn preview_suggests_next_class_and_graduation() {
        let (db, dir) = temp_db("preview");
        let grade1 = make_class(&db, 1, "Blue", 2026);
        let grade7 = make_class(&db, 7, "Blue", 2026);
        let grade2_next = make_class(&db, 2, "Blue", 2027);
        let term26 = make_term(&db, 2026);
        make_term(&db, 2027);

        seed_student(&db, "s1");
        seed_student(&db, "s2");
        enroll(&db, "s1", &grade1, &term26);
        enroll(&db, "s2", &grade7, &term26);

        let preview = PromotionService::preview(&db, 2026, 2027).unwrap();
        assert!(preview.term_ready);
        assert_eq!(preview.candidates.len(), 2);

        let junior = preview
            .candidates
            .iter()
            .find(|candidate| candidate.student_id == "s1")
            .unwrap();
        assert_eq!(junior.suggested_outcome, "PROMOTED");
        assert_eq!(
            junior.suggested_class_id.as_deref(),
            Some(grade2_next.id.as_str())
        );
        assert_eq!(junior.from_class_name, "Grade 1 Blue");

        let senior = preview
            .candidates
            .iter()
            .find(|candidate| candidate.student_id == "s2")
            .unwrap();
        assert_eq!(senior.suggested_outcome, "GRADUATED");
        assert!(senior.suggested_class_id.is_none());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_needs_the_new_years_first_term() {
        let (db, dir) = temp_db("term");
        let grade1 = make_class(&db, 1, "Blue", 2026);
        let term26 = make_term(&db, 2026);
        seed_student(&db, "s1");
        enroll(&db, "s1", &grade1, &term26);

        let preview = PromotionService::preview(&db, 2026, 2027).unwrap();
        assert!(!preview.term_ready);

        let error = PromotionService::apply(
            &db,
            ApplyPromotionInput {
                from_year: 2026,
                to_year: 2027,
                entries: vec![PromotionEntryInput {
                    student_id: "s1".into(),
                    outcome: "GRADUATED".into(),
                    to_class_id: None,
                }],
            },
            None,
        )
        .unwrap_err();
        assert_eq!(error.code, "CONFLICT");
        assert!(error.message.contains("Term 1 of 2027"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_promotes_repeats_and_graduates() {
        let (db, dir) = temp_db("apply");
        let grade1 = make_class(&db, 1, "Blue", 2026);
        let grade2 = make_class(&db, 2, "Blue", 2026);
        let grade7 = make_class(&db, 7, "Blue", 2026);
        let term26 = make_term(&db, 2026);
        let term27 = make_term(&db, 2027);
        let grade2_next = make_class(&db, 2, "Blue", 2027);
        let grade2_repeat = make_class(&db, 2, "Green", 2027);

        seed_student(&db, "s1");
        seed_student(&db, "s2");
        seed_student(&db, "s3");
        enroll(&db, "s1", &grade1, &term26);
        enroll(&db, "s2", &grade2, &term26);
        enroll(&db, "s3", &grade7, &term26);

        let result = PromotionService::apply(
            &db,
            ApplyPromotionInput {
                from_year: 2026,
                to_year: 2027,
                entries: vec![
                    PromotionEntryInput {
                        student_id: "s1".into(),
                        outcome: "PROMOTED".into(),
                        to_class_id: Some(grade2_next.id.clone()),
                    },
                    PromotionEntryInput {
                        student_id: "s2".into(),
                        outcome: "REPEATED".into(),
                        to_class_id: Some(grade2_repeat.id.clone()),
                    },
                    PromotionEntryInput {
                        student_id: "s3".into(),
                        outcome: "GRADUATED".into(),
                        to_class_id: None,
                    },
                ],
            },
            None,
        )
        .unwrap();

        assert_eq!(result.promoted, 1);
        assert_eq!(result.repeated, 1);
        assert_eq!(result.graduated, 1);

        let moved = db
            .with_conn(|conn| {
                SqliteEnrollmentRepository::new(conn).find_by_student_term("s1", &term27.id)
            })
            .unwrap()
            .unwrap();
        assert_eq!(moved.class_id, grade2_next.id);
        assert_eq!(moved.status, "ACTIVE");

        // The previous year's enrollment is kept for history but no longer active.
        for student in ["s1", "s2", "s3"] {
            let old = db
                .with_conn(|conn| {
                    SqliteEnrollmentRepository::new(conn).find_by_student_term(student, &term26.id)
                })
                .unwrap()
                .unwrap();
            assert_eq!(old.status, "INACTIVE", "{student}");
        }
        let repeated = db
            .with_conn(|conn| {
                SqliteEnrollmentRepository::new(conn).find_by_student_term("s2", &term27.id)
            })
            .unwrap()
            .unwrap();
        assert_eq!(repeated.class_id, grade2_repeat.id);
        let graduated = db
            .with_conn(|conn| {
                SqliteEnrollmentRepository::new(conn).find_by_student_term("s3", &term27.id)
            })
            .unwrap();
        assert!(graduated.is_none());

        let status: String = db
            .with_conn(|conn| {
                Ok(
                    conn.query_row("SELECT status FROM students WHERE id = 's3'", [], |row| {
                        row.get(0)
                    })?,
                )
            })
            .unwrap();
        assert_eq!(status, "GRADUATED");

        let promotions: i64 = db
            .with_conn(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM promotions", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(promotions, 3);

        // Everyone has moved, so the preview is now empty.
        assert!(PromotionService::preview(&db, 2026, 2027)
            .unwrap()
            .candidates
            .is_empty());

        // Promoting the same student twice for the same year is refused.
        let second = PromotionService::apply(
            &db,
            ApplyPromotionInput {
                from_year: 2026,
                to_year: 2027,
                entries: vec![PromotionEntryInput {
                    student_id: "s1".into(),
                    outcome: "PROMOTED".into(),
                    to_class_id: Some(grade2_next.id.clone()),
                }],
            },
            None,
        )
        .unwrap_err();
        assert_eq!(second.code, "CONFLICT", "{}", second.message);

        let _ = fs::remove_dir_all(&dir);
    }
}
