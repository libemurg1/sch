use crate::AppState;
use crate::application::enrollment_service::EnrollmentService;
use crate::domain::dto::{CreateClassInput, CreateTermInput, EnrollStudentInput};
use crate::domain::entities::{Class, Enrollment, Term};
use crate::domain::errors::AppResult;
use tauri::State;

#[tauri::command]
pub fn list_classes(state: State<'_, AppState>) -> AppResult<Vec<Class>> {
    EnrollmentService::list_classes(&state.db)
}

#[tauri::command]
pub fn create_class(state: State<'_, AppState>, input: CreateClassInput) -> AppResult<Class> {
    EnrollmentService::create_class(&state.db, input)
}

#[tauri::command]
pub fn list_terms(state: State<'_, AppState>) -> AppResult<Vec<Term>> {
    EnrollmentService::list_terms(&state.db)
}

#[tauri::command]
pub fn get_active_term(state: State<'_, AppState>) -> AppResult<Option<Term>> {
    EnrollmentService::get_active_term(&state.db)
}

#[tauri::command]
pub fn create_term(state: State<'_, AppState>, input: CreateTermInput) -> AppResult<Term> {
    EnrollmentService::create_term(&state.db, input)
}

#[tauri::command]
pub fn enroll_student(
    state: State<'_, AppState>,
    input: EnrollStudentInput,
) -> AppResult<Enrollment> {
    EnrollmentService::enroll(&state.db, input)
}

#[tauri::command]
pub fn list_enrollments(
    state: State<'_, AppState>,
    class_id: String,
    term_id: String,
) -> AppResult<Vec<Enrollment>> {
    EnrollmentService::list_by_class_term(&state.db, &class_id, &term_id)
}

#[tauri::command]
pub fn list_student_enrollments(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Enrollment>> {
    EnrollmentService::list_by_student(&state.db, &student_id)
}
