use crate::application::auth_service::AuthService;
use crate::domain::auth::{
    AuthStatus, ChangePasswordInput, CreateUserInput, User,
};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let has_users = AuthService::has_users(&state.db)?;
    let user = state.current_user()?;
    Ok(AuthStatus { has_users, user })
}

#[tauri::command]
pub fn login(state: State<'_, AppState>, input: crate::domain::auth::LoginInput) -> AppResult<User> {
    {
        let guard = state.login_guard.lock().map_err(|_| {
            crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
        })?;
        guard.check(&input.username)?;
    }

    match AuthService::login(&state.db, &input.username, &input.password) {
        Ok(user) => {
            {
                let mut guard = state.login_guard.lock().map_err(|_| {
                    crate::domain::errors::AppError::new(
                        "LOCKED",
                        "Application state lock poisoned",
                    )
                })?;
                guard.record_success(&input.username);
            }
            *state.session.lock().map_err(|_| {
                crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
            })? = Some(user.clone());
            Ok(user)
        }
        Err(error) => {
            {
                let mut guard = state.login_guard.lock().map_err(|_| {
                    crate::domain::errors::AppError::new(
                        "LOCKED",
                        "Application state lock poisoned",
                    )
                })?;
                guard.record_failure(&input.username);
            }
            Err(error)
        }
    }
}

#[tauri::command]
pub fn logout(state: State<'_, AppState>) -> AppResult<()> {
    // Logout is always allowed so a stuck session can be cleared.
    *state.session.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    Ok(())
}

#[tauri::command]
pub fn get_current_user(state: State<'_, AppState>) -> AppResult<Option<User>> {
    state.current_user()
}

#[tauri::command]
pub fn create_user(
    state: State<'_, AppState>,
    input: CreateUserInput,
) -> AppResult<User> {
    let authenticated = state.current_user()?.is_some();
    AuthService::create_user(&state.db, &input, authenticated)
}

#[tauri::command]
pub fn list_users(state: State<'_, AppState>) -> AppResult<Vec<User>> {
    let _user = state.require_auth()?;
    AuthService::list_users(&state.db)
}

#[tauri::command]
pub fn change_password(
    state: State<'_, AppState>,
    input: ChangePasswordInput,
) -> AppResult<()> {
    let user = state.require_auth()?;
    AuthService::change_password(&state.db, &user.id, &input)
}

#[tauri::command]
pub fn set_user_active(
    state: State<'_, AppState>,
    user_id: String,
    is_active: bool,
) -> AppResult<()> {
    let actor = state.require_auth()?;
    if actor.id == user_id && !is_active {
        return Err(crate::domain::errors::AppError::validation(
            "You cannot deactivate your own account",
        ));
    }
    AuthService::set_active(&state.db, &user_id, is_active)
}

#[tauri::command]
pub fn update_user(
    state: State<'_, AppState>,
    user_id: String,
    username: String,
    display_name: String,
) -> AppResult<User> {
    let actor = state.require_auth()?;
    let updated = AuthService::update_user(&state.db, &user_id, &username, &display_name)?;

    // The active session holds its own copy of the user, so keep it in step
    // when someone edits their own account.
    if actor.id == updated.id {
        *state.session.lock().map_err(|_| {
            crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
        })? = Some(updated.clone());
    }

    Ok(updated)
}

#[tauri::command]
pub fn delete_user(state: State<'_, AppState>, user_id: String) -> AppResult<()> {
    let actor = state.require_auth()?;
    if actor.id == user_id {
        return Err(crate::domain::errors::AppError::validation(
            "You cannot delete your own account",
        ));
    }
    AuthService::delete_user(&state.db, &user_id)
}
