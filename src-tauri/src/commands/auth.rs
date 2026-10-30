use crate::application::auth_service::AuthService;
use crate::domain::auth::{AuthStatus, ChangePasswordInput, CreateUserInput, User, UserRole};
use crate::domain::errors::AppResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn get_auth_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let has_users = AuthService::has_users(&state.db)?;
    let user = state.current_user()?;
    let session_restore_notice = state.take_session_restore_notice()?;
    Ok(AuthStatus {
        has_users,
        user,
        session_restore_notice,
    })
}

#[tauri::command]
pub fn login(
    state: State<'_, AppState>,
    input: crate::domain::auth::LoginInput,
) -> AppResult<User> {
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
            if input.remember_me {
                let token = AuthService::create_remembered_session(&state.db, &user.id)?;
                if let Err(error) = crate::infrastructure::remembered_session::store(
                    &state.remembered_session_path,
                    &token,
                ) {
                    AuthService::revoke_remembered_session(&state.db, &token)?;
                    return Err(error);
                }
                *state.remembered_session_token.lock().map_err(|_| {
                    crate::domain::errors::AppError::new(
                        "LOCKED",
                        "Application state lock poisoned",
                    )
                })? = Some(token);
            } else {
                let previous_token = state
                    .remembered_session_token
                    .lock()
                    .map_err(|_| {
                        crate::domain::errors::AppError::new(
                            "LOCKED",
                            "Application state lock poisoned",
                        )
                    })?
                    .clone();
                if let Some(token) = &previous_token {
                    AuthService::revoke_remembered_session(&state.db, token)?;
                }
                crate::infrastructure::remembered_session::clear(&state.remembered_session_path)?;
                *state.remembered_session_token.lock().map_err(|_| {
                    crate::domain::errors::AppError::new(
                        "LOCKED",
                        "Application state lock poisoned",
                    )
                })? = None;
            }
            *state.session_restore_notice.lock().map_err(|_| {
                crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
            })? = None;
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
    let token = state
        .remembered_session_token
        .lock()
        .map_err(|_| {
            crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
        })?
        .clone();
    crate::infrastructure::remembered_session::clear(&state.remembered_session_path)?;
    if let Some(token) = token {
        AuthService::revoke_remembered_session(&state.db, &token)?;
    }
    *state.session.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    *state.remembered_session_token.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    Ok(())
}

#[tauri::command]
pub fn get_current_user(state: State<'_, AppState>) -> AppResult<Option<User>> {
    state.current_user()
}

#[tauri::command]
pub fn create_user(state: State<'_, AppState>, input: CreateUserInput) -> AppResult<User> {
    let authenticated = AuthService::has_users(&state.db)?;
    if authenticated {
        state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
    }
    AuthService::create_user(&state.db, &input, authenticated)
}

#[tauri::command]
pub fn list_users(state: State<'_, AppState>) -> AppResult<Vec<User>> {
    let _user = state.require_auth()?;
    AuthService::list_users(&state.db)
}

#[tauri::command]
pub fn change_password(state: State<'_, AppState>, input: ChangePasswordInput) -> AppResult<()> {
    let user = state.require_auth()?;
    AuthService::change_password(&state.db, &user.id, &input)?;
    *state.remembered_session_token.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    crate::infrastructure::remembered_session::clear(&state.remembered_session_path)
}

#[tauri::command]
pub fn create_administrator_recovery_code(state: State<'_, AppState>) -> AppResult<String> {
    let user = state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
    AuthService::create_administrator_recovery_code(&state.db, &user.id)
}

#[tauri::command]
pub fn administrator_recovery_configured(state: State<'_, AppState>) -> AppResult<bool> {
    let user = state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
    AuthService::administrator_recovery_configured(&state.db, &user.id)
}

#[tauri::command]
pub fn recover_administrator(
    state: State<'_, AppState>,
    username: String,
    recovery_code: String,
    new_password: String,
) -> AppResult<String> {
    let replacement_code =
        AuthService::recover_administrator(&state.db, &username, &recovery_code, &new_password)?;
    *state.session.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    *state.remembered_session_token.lock().map_err(|_| {
        crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
    })? = None;
    crate::infrastructure::remembered_session::clear(&state.remembered_session_path)?;
    state
        .login_guard
        .lock()
        .map_err(|_| {
            crate::domain::errors::AppError::new("LOCKED", "Application state lock poisoned")
        })?
        .record_success(&username);
    Ok(replacement_code)
}

#[tauri::command]
pub fn set_user_active(
    state: State<'_, AppState>,
    user_id: String,
    is_active: bool,
) -> AppResult<()> {
    let actor = state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
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
    role: UserRole,
) -> AppResult<User> {
    let actor = state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
    let updated = AuthService::update_user(&state.db, &user_id, &username, &display_name, role)?;

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
    let actor = state.require_permission(crate::domain::auth::Permission::ManageUsers)?;
    if actor.id == user_id {
        return Err(crate::domain::errors::AppError::validation(
            "You cannot delete your own account",
        ));
    }
    AuthService::delete_user(&state.db, &user_id)
}
