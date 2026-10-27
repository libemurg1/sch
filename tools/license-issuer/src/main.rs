use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::Argon2;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::NaiveDate;
use ed25519_dalek::{Signer, SigningKey};
use rand_core::OsRng;
use serde::Serialize;
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

const PRODUCT_ID: &str = "com.schoolfees.primaryschool";
const KEY_ENTROPY: &[u8] = b"com.schoolfees.primaryschool.license-issuer-key.v1";
const APP_PUBLIC_KEY_HEX: &str = include_str!("../../../src-tauri/license-public-key.txt");

#[derive(Serialize)]
struct LicensePayload<'a> {
    version: u8,
    product: &'static str,
    activation_id: &'a str,
    school: &'a str,
    starts_on: &'a str,
    expires_on: &'a str,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("License issuer error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("init") => {
            let key_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or(default_key_path()?);
            initialize_key(&key_path)?;
        }
        Some("public-key") => {
            let key_path = args
                .next()
                .map(PathBuf::from)
                .unwrap_or(default_key_path()?);
            let protected = fs::read(&key_path)?;
            let signing_key = read_signing_key(&protected)?;
            ensure_trusted_key(&signing_key)?;
            println!("{}", encode_hex(&signing_key.verifying_key().to_bytes()));
        }
        Some("issue") => issue_license(args.collect())?,
        Some("export-key") => export_key(args.collect())?,
        Some("import-key") => import_key(args.collect())?,
        _ => {
            eprintln!(
                "Usage:\n  sfs-license-issuer init [protected-key-path]\n  \
                 sfs-license-issuer public-key [protected-key-path]\n  \
                 sfs-license-issuer issue --activation-id ID --school NAME \
                 --starts-on YYYY-MM-DD --expires-on YYYY-MM-DD [--key protected-key-path]\n  \
                 sfs-license-issuer export-key BACKUP-FILE [--key protected-key-path]\n  \
                 sfs-license-issuer import-key BACKUP-FILE [protected-key-path]"
            );
            std::process::exit(2);
        }
    }
    Ok(())
}

fn export_key(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (path, key_path) = parse_key_file_args(args, true)?;
    if path.exists() {
        return Err(format!("Refusing to overwrite existing file: {}", path.display()).into());
    }
    let protected = fs::read(&key_path)?;
    let signing_key = read_signing_key(&protected)?;
    let password = rpassword::prompt_password("Backup passphrase: ")?;
    let confirmation = rpassword::prompt_password("Confirm passphrase: ")?;
    if password.len() < 12 {
        return Err("Use a backup passphrase with at least 12 characters".into());
    }
    if password != confirmation {
        return Err("Passphrases do not match".into());
    }

    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    use rand_core::RngCore;
    OsRng.fill_bytes(&mut salt);
    OsRng.fill_bytes(&mut nonce);
    let mut encryption_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut encryption_key)
        .map_err(|error| format!("Could not derive the encryption key: {error}"))?;
    let cipher = Aes256Gcm::new_from_slice(&encryption_key)?;
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), signing_key.to_bytes().as_slice())
        .map_err(|_| "Could not encrypt signing key backup")?;

    let mut contents = b"SFSKEY1".to_vec();
    contents.extend_from_slice(&salt);
    contents.extend_from_slice(&nonce);
    contents.extend_from_slice(&encrypted);
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    output.write_all(&contents)?;
    output.sync_all()?;
    println!(
        "Encrypted signing-key backup created at: {}",
        path.display()
    );
    Ok(())
}

fn import_key(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let (backup_path, key_path) = parse_key_file_args(args, false)?;
    if key_path.exists() {
        return Err(format!(
            "A signing key already exists at {}. Refusing to overwrite it.",
            key_path.display()
        )
        .into());
    }
    let contents = fs::read(&backup_path)?;
    if contents.len() != 7 + 16 + 12 + 32 + 16 || !contents.starts_with(b"SFSKEY1") {
        return Err("The encrypted signing-key backup has an invalid format".into());
    }
    let salt = &contents[7..23];
    let nonce = &contents[23..35];
    let encrypted = &contents[35..];
    let password = rpassword::prompt_password("Backup passphrase: ")?;
    let mut encryption_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut encryption_key)
        .map_err(|error| format!("Could not derive the encryption key: {error}"))?;
    let cipher = Aes256Gcm::new_from_slice(&encryption_key)?;
    let signing_key = cipher
        .decrypt(Nonce::from_slice(nonce), encrypted)
        .map_err(|_| "Wrong passphrase or damaged signing-key backup")?;
    let signing_key: [u8; 32] = signing_key
        .try_into()
        .map_err(|_| "The signing-key backup contained an invalid key")?;
    let protected = protect(&signing_key)?;
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&key_path)?;
    output.write_all(&protected)?;
    output.sync_all()?;
    println!("Signing key restored to: {}", key_path.display());
    Ok(())
}

fn parse_key_file_args(
    args: Vec<String>,
    require_existing_default_key: bool,
) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let mut args = args.into_iter();
    let file = args.next().ok_or("A backup file path is required")?;
    let mut key_path = default_key_path()?;
    while let Some(flag) = args.next() {
        if flag != "--key" {
            return Err(format!("Unknown option: {flag}").into());
        }
        key_path = PathBuf::from(args.next().ok_or("Missing path after --key")?);
    }
    if args.next().is_some() {
        return Err("Too many arguments".into());
    }
    if require_existing_default_key && !key_path.exists() {
        return Err(format!("Signing key not found at {}", key_path.display()).into());
    }
    Ok((PathBuf::from(file), key_path))
}

fn default_key_path() -> Result<PathBuf, Box<dyn Error>> {
    let local_app_data = env::var_os("LOCALAPPDATA")
        .ok_or("LOCALAPPDATA is not available; specify a protected key path")?;
    Ok(PathBuf::from(local_app_data)
        .join("SFS-License-Issuer")
        .join("signing-key.dpapi"))
}

fn initialize_key(path: &Path) -> Result<(), Box<dyn Error>> {
    if path.exists() {
        return Err(format!(
            "A signing key already exists at {}. Refusing to overwrite it.",
            path.display()
        )
        .into());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let signing_key = SigningKey::generate(&mut OsRng);
    let protected = protect(&signing_key.to_bytes())?;
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(&protected)?;
    file.sync_all()?;

    println!("Created DPAPI-protected signing key at: {}", path.display());
    println!(
        "Public verification key (hex): {}",
        encode_hex(&signing_key.verifying_key().to_bytes())
    );
    println!(
        "Keep this key file backed up securely. Do not include it in app builds or source control."
    );
    Ok(())
}

fn issue_license(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut activation_id = None;
    let mut school = None;
    let mut starts_on = None;
    let mut expires_on = None;
    let mut key_path = default_key_path()?;

    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| format!("Missing value for {flag}"))?;
        match flag.as_str() {
            "--activation-id" => activation_id = Some(value),
            "--school" => school = Some(value),
            "--starts-on" => starts_on = Some(value),
            "--expires-on" => expires_on = Some(value),
            "--key" => key_path = PathBuf::from(value),
            _ => return Err(format!("Unknown option: {flag}").into()),
        }
    }

    let activation_id = activation_id.ok_or("--activation-id is required")?;
    let school = school.ok_or("--school is required")?;
    let starts_on = starts_on.ok_or("--starts-on is required")?;
    let expires_on = expires_on.ok_or("--expires-on is required")?;
    let start_date = parse_date(&starts_on)?;
    let expiry_date = parse_date(&expires_on)?;
    if school.trim().is_empty() {
        return Err("School name cannot be empty".into());
    }
    if expiry_date < start_date {
        return Err("Expiry date must be on or after the start date".into());
    }
    if activation_id.trim().is_empty() {
        return Err("Activation ID cannot be empty".into());
    }

    let protected = fs::read(&key_path)
        .map_err(|error| format!("Cannot read signing key at {}: {error}", key_path.display()))?;
    let signing_key = read_signing_key(&protected)?;
    ensure_trusted_key(&signing_key)?;
    let payload = LicensePayload {
        version: 1,
        product: PRODUCT_ID,
        activation_id: activation_id.trim(),
        school: school.trim(),
        starts_on: &starts_on,
        expires_on: &expires_on,
    };
    let payload_bytes = serde_json::to_vec(&payload)?;
    let signature = signing_key.sign(&payload_bytes);
    println!(
        "sfs1.{}.{}",
        URL_SAFE_NO_PAD.encode(payload_bytes),
        URL_SAFE_NO_PAD.encode(signature.to_bytes())
    );
    Ok(())
}

fn read_signing_key(protected: &[u8]) -> Result<SigningKey, Box<dyn Error>> {
    let key_bytes = unprotect(protected)?;
    let key_bytes: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "The protected signing key has an invalid length")?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

fn ensure_trusted_key(signing_key: &SigningKey) -> Result<(), Box<dyn Error>> {
    let public_key_hex = encode_hex(&signing_key.verifying_key().to_bytes());
    if public_key_hex != APP_PUBLIC_KEY_HEX.trim() {
        return Err(
            "The signing key does not match the public key trusted by this app build. \
             Check the issuer key or update src-tauri/license-public-key.txt and rebuild the app."
                .into(),
        );
    }
    Ok(())
}

fn parse_date(value: &str) -> Result<NaiveDate, Box<dyn Error>> {
    Ok(NaiveDate::parse_from_str(value, "%Y-%m-%d")?)
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(windows)]
fn protect(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::ffi::c_void;
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: KEY_ENTROPY.len() as u32,
        pbData: KEY_ENTROPY.as_ptr() as *mut u8,
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
        return Err(std::io::Error::last_os_error().into());
    }
    let protected =
        unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(protected)
}

#[cfg(windows)]
fn unprotect(data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    use std::ffi::c_void;
    use std::slice;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let input = CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    };
    let entropy = CRYPT_INTEGER_BLOB {
        cbData: KEY_ENTROPY.len() as u32,
        pbData: KEY_ENTROPY.as_ptr() as *mut u8,
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
        return Err(std::io::Error::last_os_error().into());
    }
    let plain = unsafe { slice::from_raw_parts(output.pbData, output.cbData as usize) }.to_vec();
    unsafe {
        LocalFree(output.pbData as *mut c_void);
    }
    Ok(plain)
}

#[cfg(not(windows))]
fn protect(_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    Err("The license issuer currently requires Windows DPAPI".into())
}

#[cfg(not(windows))]
fn unprotect(_data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
    Err("The license issuer currently requires Windows DPAPI".into())
}
