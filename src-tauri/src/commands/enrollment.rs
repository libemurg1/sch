use crate::AppState;
use crate::application::enrollment_service::EnrollmentService;
use crate::domain::dto::{
    CreateClassInput, CreateTermInput, EnrollStudentInput, UpdateClassInput, UpdateTermInput,
};
use crate::domain::entities::{Class, Enrollment, Term};
use crate::domain::errors::AppResult;
use tauri::State;

#[tauri::command]
pub fn list_classes(state: State<'_, AppState>) -> AppResult<Vec<Class>> {
    let _user = state.require_auth()?;
    EnrollmentService::list_classes(&state.db)
}

#[tauri::command]
pub fn create_class(state: State<'_, AppState>, input: CreateClassInput) -> AppResult<Class> {
    let _user = state.require_auth()?;
    EnrollmentService::create_class(&state.db, input)
}

#[tauri::command]
pub fn update_class(state: State<'_, AppState>, input: UpdateClassInput) -> AppResult<Class> {
    let _user = state.require_auth()?;
    EnrollmentService::update_class(&state.db, input)
}

#[tauri::command]
pub fn delete_class(state: State<'_, AppState>, class_id: String) -> AppResult<()> {
    let _user = state.require_auth()?;
    EnrollmentService::delete_class(&state.db, &class_id)
}

#[tauri::command]
pub fn list_terms(state: State<'_, AppState>) -> AppResult<Vec<Term>> {
    let _user = state.require_auth()?;
    EnrollmentService::list_terms(&state.db)
}

#[tauri::command]
pub fn get_active_term(state: State<'_, AppState>) -> AppResult<Option<Term>> {
    let _user = state.require_auth()?;
    EnrollmentService::get_active_term(&state.db)
}

#[tauri::command]
pub fn create_term(state: State<'_, AppState>, input: CreateTermInput) -> AppResult<Term> {
    let _user = state.require_auth()?;
    EnrollmentService::create_term(&state.db, input)
}

#[tauri::command]
pub fn update_term(state: State<'_, AppState>, input: UpdateTermInput) -> AppResult<Term> {
    let _user = state.require_auth()?;
    EnrollmentService::update_term(&state.db, input)
}

#[tauri::command]
pub fn delete_term(state: State<'_, AppState>, term_id: String) -> AppResult<()> {
    let _user = state.require_auth()?;
    EnrollmentService::delete_term(&state.db, &term_id)
}

#[tauri::command]
pub fn enroll_student(
    state: State<'_, AppState>,
    input: EnrollStudentInput,
) -> AppResult<Enrollment> {
    let _user = state.require_auth()?;
    EnrollmentService::enroll(&state.db, input)
}

#[tauri::command]
pub fn list_enrollments(
    state: State<'_, AppState>,
    class_id: String,
    term_id: String,
) -> AppResult<Vec<Enrollment>> {
    let _user = state.require_auth()?;
    EnrollmentService::list_by_class_term(&state.db, &class_id, &term_id)
}

#[tauri::command]
pub fn list_student_enrollments(
    state: State<'_, AppState>,
    student_id: String,
) -> AppResult<Vec<Enrollment>> {
    let _user = state.require_auth()?;
    EnrollmentService::list_by_student(&state.db, &student_id)
}
