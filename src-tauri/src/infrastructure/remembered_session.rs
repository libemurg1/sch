use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::protected_key;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn load(path: &Path) -> AppResult<Option<String>> {
    if !path.exists() {
        return Ok(None);
    }

    let protected = fs::read(path)?;
    let token = protected_key::unprotect_session_token(&protected)?;
    let token = String::from_utf8(token).map_err(|error| {
        AppError::new(
            "SESSION_RESTORE_FAILED",
            format!("The remembered sign-in token is invalid: {error}"),
        )
    })?;
    if token.is_empty() {
        return Err(AppError::new(
            "SESSION_RESTORE_FAILED",
            "The remembered sign-in token is empty",
        ));
    }
    Ok(Some(token))
}

pub fn store(path: &Path, token: &str) -> AppResult<()> {
    let protected = protected_key::protect_session_token(token.as_bytes())?;
    let temporary_path = temporary_path(path);
    let write_result = (|| -> AppResult<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        file.write_all(&protected)?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&temporary_path);
        return Err(error);
    }

    if path.exists() {
        if let Err(error) = fs::remove_file(path) {
            let _ = fs::remove_file(&temporary_path);
            return Err(AppError::from(error));
        }
    }
    if let Err(error) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(AppError::from(error));
    }
    Ok(())
}

pub fn clear(path: &Path) -> AppResult<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()))
}

#[cfg(test)]
mod tests {
    use super::{clear, load, store};

    #[test]
    fn remembered_token_is_protected_on_disk_and_can_be_cleared() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("remembered-session.bin");
        let token = "opaque-session-token";

        store(&path, token).unwrap();
        assert_eq!(load(&path).unwrap().as_deref(), Some(token));

        clear(&path).unwrap();
        assert!(load(&path).unwrap().is_none());
    }
}
