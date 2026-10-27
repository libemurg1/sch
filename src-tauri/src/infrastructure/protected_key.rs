use crate::domain::errors::{AppError, AppResult};

#[cfg(windows)]
pub fn protect_key(key: &[u8]) -> AppResult<Vec<u8>> {
    protect_data(key, b"com.schoolfees.primaryschool.db-key.v1")
}

#[cfg(windows)]
pub fn protect_license_data(data: &[u8]) -> AppResult<Vec<u8>> {
    protect_data(data, b"com.schoolfees.primaryschool.license-state.v1")
}

#[cfg(windows)]
fn protect_data(key: &[u8], entropy_bytes: &[u8]) -> AppResult<Vec<u8>> {
    use std::ffi::c_void;
    use std::slice;
    use windows_sys::Win32::Foundation::{GetLastError, LocalFree};
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: key.len() as u32,
        pbData: key.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: entropy_bytes.len() as u32,
        pbData: entropy_bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let succeeded = unsafe {
        CryptProtectData(
            &input,
            std::ptr::null(),
            &entropy,
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if succeeded == 0 {
        return Err(AppError::new(
            "KEY_PROTECTION_FAILED",
            format!("Windows could not protect the database key: {}", unsafe {
                GetLastError()
            }),
        ));
    }

    let protected =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(protected)
}

#[cfg(windows)]
pub fn unprotect_key(protected_key: &[u8]) -> AppResult<Vec<u8>> {
    unprotect_data(protected_key, b"com.schoolfees.primaryschool.db-key.v1")
}

#[cfg(windows)]
pub fn unprotect_license_data(protected_data: &[u8]) -> AppResult<Vec<u8>> {
    unprotect_data(
        protected_data,
        b"com.schoolfees.primaryschool.license-state.v1",
    )
}

#[cfg(windows)]
fn unprotect_data(protected_key: &[u8], entropy_bytes: &[u8]) -> AppResult<Vec<u8>> {
    use std::ffi::c_void;
    use std::slice;
    use windows_sys::Win32::Foundation::{GetLastError, LocalFree};
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: protected_key.len() as u32,
        pbData: protected_key.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: entropy_bytes.len() as u32,
        pbData: entropy_bytes.as_ptr() as *mut u8,
    };
    let mut output = CRYPT_INTEGER_BLOB::default();
    let succeeded = unsafe {
        CryptUnprotectData(
            &input,
            std::ptr::null_mut(),
            &entropy,
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut output,
        )
    };
    if succeeded == 0 {
        return Err(AppError::new(
            "KEY_UNPROTECTION_FAILED",
            format!("Windows could not unlock the database key: {}", unsafe {
                GetLastError()
            }),
        ));
    }

    let key = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(key)
}

#[cfg(not(windows))]
pub fn protect_key(_key: &[u8]) -> AppResult<Vec<u8>> {
    Err(AppError::new(
        "UNSUPPORTED_PLATFORM",
        "Encrypted local database storage is currently supported on Windows only",
    ))
}

#[cfg(not(windows))]
pub fn unprotect_key(_protected_key: &[u8]) -> AppResult<Vec<u8>> {
    Err(AppError::new(
        "UNSUPPORTED_PLATFORM",
        "Encrypted local database storage is currently supported on Windows only",
    ))
}

#[cfg(not(windows))]
pub fn protect_license_data(_data: &[u8]) -> AppResult<Vec<u8>> {
    Err(AppError::new(
        "UNSUPPORTED_PLATFORM",
        "Protected license state is currently supported on Windows only",
    ))
}

#[cfg(not(windows))]
pub fn unprotect_license_data(_protected_data: &[u8]) -> AppResult<Vec<u8>> {
    Err(AppError::new(
        "UNSUPPORTED_PLATFORM",
        "Protected license state is currently supported on Windows only",
    ))
}
