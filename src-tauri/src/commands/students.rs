use crate::application::student_service::StudentService;
use crate::domain::dto::{RegisterStudentInput, UpdateStudentInput, UpdateStudentStatusInput};
use crate::domain::entities::Student;
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn list_students(state: State<'_, AppState>) -> AppResult<Vec<Student>> {
    let _user = state.require_auth()?;
    StudentService::list(&state.db)
}

#[tauri::command]
pub fn list_students_by_status(
    state: State<'_, AppState>,
    status: String,
) -> AppResult<Vec<Student>> {
    let _user = state.require_auth()?;
    StudentService::list_by_status(&state.db, &status)
}

#[tauri::command]
pub fn register_student(
    state: State<'_, AppState>,
    input: RegisterStudentInput,
) -> AppResult<Student> {
    let _user = state.require_write_auth()?;
    StudentService::register(&state.db, input)
}

#[tauri::command]
pub fn get_student(state: State<'_, AppState>, id: String) -> AppResult<Option<Student>> {
    let _user = state.require_auth()?;
    StudentService::find(&state.db, &id)
}

#[tauri::command]
pub fn update_student(state: State<'_, AppState>, input: UpdateStudentInput) -> AppResult<Student> {
    let _user = state.require_write_auth()?;
    StudentService::update_details(&state.db, input)
}

#[tauri::command]
pub fn update_student_status(
    state: State<'_, AppState>,
    input: UpdateStudentStatusInput,
) -> AppResult<()> {
    let _user = state.require_write_auth()?;
    StudentService::update_status(&state.db, &input.student_id, &input.status)
}
