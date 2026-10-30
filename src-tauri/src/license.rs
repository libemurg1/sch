use crate::domain::errors::{AppError, AppResult};
use crate::infrastructure::protected_key;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, NaiveDate, Utc};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

const PRODUCT_ID: &str = "com.schoolfees.primaryschool";
const TRIAL_DAYS: i64 = 14;
const GRACE_DAYS: i64 = 5;
const PUBLIC_KEY_HEX: &str = include_str!("../license-public-key.txt");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePayload {
    pub version: u8,
    pub product: String,
    pub activation_id: String,
    pub school: String,
    pub starts_on: String,
    pub expires_on: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseStatus {
    pub state: String,
    pub activation_id: String,
    pub trial_days_remaining: i64,
    pub trial_started_on: String,
    pub trial_ends_on: String,
    pub license_school: Option<String>,
    pub starts_on: Option<String>,
    pub expires_on: Option<String>,
    pub grace_days_remaining: i64,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LicenseStore {
    activation_id: String,
    trial_started_on: String,
    highest_seen_on: String,
    license_key: Option<String>,
}

pub struct Licensing {
    path: PathBuf,
    store: Mutex<LicenseStore>,
}

impl Licensing {
    pub fn open(app_dir: &Path) -> AppResult<Self> {
        let path = app_dir.join("license.state");
        let store = if path.exists() {
            let protected = fs::read(&path)?;
            let bytes = protected_key::unprotect_license_data(&protected)?;
            serde_json::from_slice(&bytes).map_err(|error| {
                AppError::new(
                    "LICENSE_STATE_INVALID",
                    format!("The local license state could not be read: {error}"),
                )
            })?
        } else {
            let today = Utc::now().date_naive();
            let today = format_date(today);
            let store = LicenseStore {
                activation_id: uuid::Uuid::new_v4().to_string(),
                trial_started_on: today.clone(),
                highest_seen_on: today,
                license_key: None,
            };
            persist(&path, &store)?;
            store
        };
        parse_date(&store.trial_started_on)?;
        parse_date(&store.highest_seen_on)?;
        uuid::Uuid::parse_str(&store.activation_id).map_err(|error| {
            AppError::new(
                "LICENSE_STATE_INVALID",
                format!("The local device activation ID is invalid: {error}"),
            )
        })?;

        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    pub fn status(&self) -> AppResult<LicenseStatus> {
        let mut store = self.lock()?;
        let today = effective_today(&mut store)?;
        persist(&self.path, &store)?;
        status_for(&store, today)
    }

    pub fn activate(&self, license_key: &str) -> AppResult<LicenseStatus> {
        let payload = verify_license(license_key)?;
        let mut store = self.lock()?;
        if payload.activation_id != store.activation_id {
            return Err(AppError::validation(
                "This license key was issued for a different device",
            ));
        }
        let today = effective_today(&mut store)?;
        let starts_on = parse_date(&payload.starts_on)?;
        let expires_on = parse_date(&payload.expires_on)?;
        if expires_on < starts_on {
            return Err(AppError::validation(
                "The license expiry date must be on or after its start date",
            ));
        }
        if today > expires_on + Duration::days(GRACE_DAYS) {
            return Err(AppError::validation("This license key has expired"));
        }

        store.license_key = Some(license_key.trim().to_string());
        persist(&self.path, &store)?;
        status_for(&store, today)
    }

    pub fn ensure_writable(&self) -> AppResult<()> {
        let status = self.status()?;
        if status.state == "readOnly" {
            return Err(AppError::new(
                "LICENSE_READ_ONLY",
                "The trial or license has expired. Viewing data and backups remain available; activate a valid license to make changes.",
            ));
        }
        Ok(())
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, LicenseStore>> {
        self.store
            .lock()
            .map_err(|_| AppError::new("LICENSE_STATE_LOCKED", "License state lock is poisoned"))
    }
}

fn effective_today(store: &mut LicenseStore) -> AppResult<NaiveDate> {
    let system_today = Utc::now().date_naive();
    let highest_seen = parse_date(&store.highest_seen_on)?;
    let effective = system_today.max(highest_seen);
    if effective > highest_seen {
        store.highest_seen_on = format_date(effective);
    }
    Ok(effective)
}

fn status_for(store: &LicenseStore, today: NaiveDate) -> AppResult<LicenseStatus> {
    let trial_start = parse_date(&store.trial_started_on)?;
    let trial_end = trial_start + Duration::days(TRIAL_DAYS);
    let trial_elapsed = (today - trial_start).num_days().max(0);
    let trial_remaining = (TRIAL_DAYS - trial_elapsed).max(0);

    if let Some(key) = &store.license_key {
        let payload = verify_license(key)?;
        let starts_on = parse_date(&payload.starts_on)?;
        let expires_on = parse_date(&payload.expires_on)?;
        let expiry_grace_ends = expires_on + Duration::days(GRACE_DAYS);

        if today < starts_on {
            let state = if trial_elapsed < TRIAL_DAYS {
                "trial"
            } else if trial_elapsed < TRIAL_DAYS + GRACE_DAYS {
                "grace"
            } else {
                "readOnly"
            };
            return Ok(LicenseStatus {
                state: state.to_string(),
                activation_id: store.activation_id.clone(),
                trial_days_remaining: trial_remaining,
                trial_started_on: format_date(trial_start),
                trial_ends_on: format_date(trial_end),
                license_school: Some(payload.school),
                starts_on: Some(payload.starts_on),
                expires_on: Some(payload.expires_on),
                grace_days_remaining: if state == "grace" {
                    TRIAL_DAYS + GRACE_DAYS - trial_elapsed
                } else {
                    0
                },
                message: "The license is not active yet.".to_string(),
            });
        }
        if today <= expires_on {
            return Ok(LicenseStatus {
                state: "active".to_string(),
                activation_id: store.activation_id.clone(),
                trial_days_remaining: 0,
                trial_started_on: format_date(trial_start),
                trial_ends_on: format_date(trial_end),
                license_school: Some(payload.school),
                starts_on: Some(payload.starts_on),
                expires_on: Some(payload.expires_on),
                grace_days_remaining: 0,
                message: "The annual license is active.".to_string(),
            });
        }
        if today <= expiry_grace_ends {
            let grace_remaining = (expiry_grace_ends - today).num_days() + 1;
            return Ok(LicenseStatus {
                state: "grace".to_string(),
                activation_id: store.activation_id.clone(),
                trial_days_remaining: 0,
                trial_started_on: format_date(trial_start),
                trial_ends_on: format_date(trial_end),
                license_school: Some(payload.school),
                starts_on: Some(payload.starts_on),
                expires_on: Some(payload.expires_on),
                grace_days_remaining: grace_remaining,
                message: "The annual license has expired. Renew during the grace period to continue making changes.".to_string(),
            });
        }
        return Ok(LicenseStatus {
            state: "readOnly".to_string(),
            activation_id: store.activation_id.clone(),
            trial_days_remaining: 0,
            trial_started_on: format_date(trial_start),
            trial_ends_on: format_date(trial_end),
            license_school: Some(payload.school),
            starts_on: Some(payload.starts_on),
            expires_on: Some(payload.expires_on),
            grace_days_remaining: 0,
            message: "The license and grace period have expired. The app is read-only.".to_string(),
        });
    }

    if today < trial_end {
        Ok(LicenseStatus {
            state: "trial".to_string(),
            activation_id: store.activation_id.clone(),
            trial_days_remaining: (trial_end - today).num_days(),
            trial_started_on: format_date(trial_start),
            trial_ends_on: format_date(trial_end),
            license_school: None,
            starts_on: None,
            expires_on: None,
            grace_days_remaining: 0,
            message: "The 14-day trial is active.".to_string(),
        })
    } else {
        let grace_ends = trial_end + Duration::days(GRACE_DAYS);
        if today < grace_ends {
            Ok(LicenseStatus {
                state: "grace".to_string(),
                activation_id: store.activation_id.clone(),
                trial_days_remaining: 0,
                trial_started_on: format_date(trial_start),
                trial_ends_on: format_date(trial_end),
                license_school: None,
                starts_on: None,
                expires_on: None,
                grace_days_remaining: (grace_ends - today).num_days(),
                message: "The trial has ended. The app will become read-only when the grace period finishes.".to_string(),
            })
        } else {
            Ok(LicenseStatus {
                state: "readOnly".to_string(),
                activation_id: store.activation_id.clone(),
                trial_days_remaining: 0,
                trial_started_on: format_date(trial_start),
                trial_ends_on: format_date(trial_end),
                license_school: None,
                starts_on: None,
                expires_on: None,
                grace_days_remaining: 0,
                message: "The trial and grace period have expired. The app is read-only."
                    .to_string(),
            })
        }
    }
}

pub fn verify_license(key: &str) -> AppResult<LicensePayload> {
    let public_bytes = decode_hex(PUBLIC_KEY_HEX)?;
    let public_bytes: [u8; 32] = public_bytes.try_into().map_err(|_| {
        AppError::new(
            "LICENSE_CONFIG_INVALID",
            "The application license verification key is not configured",
        )
    })?;
    verify_license_with_key(key, &public_bytes)
}

fn verify_license_with_key(key: &str, public_bytes: &[u8; 32]) -> AppResult<LicensePayload> {
    let mut parts = key.trim().split('.');
    let prefix = parts.next();
    let payload_part = parts.next();
    let signature_part = parts.next();
    if prefix != Some("sfs1") || parts.next().is_some() {
        return Err(AppError::validation("The license key format is invalid"));
    }
    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_part.unwrap_or_default())
        .map_err(|_| AppError::validation("The license key payload is invalid"))?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(signature_part.unwrap_or_default())
        .map_err(|_| AppError::validation("The license key signature is invalid"))?;
    let public_key = VerifyingKey::from_bytes(public_bytes).map_err(|error| {
        AppError::new(
            "LICENSE_CONFIG_INVALID",
            format!("The application license verification key is invalid: {error}"),
        )
    })?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| AppError::validation("The license key signature is invalid"))?;
    public_key
        .verify(&payload_bytes, &signature)
        .map_err(|_| AppError::validation("The license key signature could not be verified"))?;
    let payload: LicensePayload = serde_json::from_slice(&payload_bytes)
        .map_err(|_| AppError::validation("The signed license data is invalid"))?;
    if payload.version != 1 || payload.product != PRODUCT_ID {
        return Err(AppError::validation(
            "This license was issued for a different product or version",
        ));
    }
    if payload.school.trim().is_empty() {
        return Err(AppError::validation(
            "The license does not contain a school name",
        ));
    }
    parse_date(&payload.starts_on)?;
    parse_date(&payload.expires_on)?;
    Ok(payload)
}

fn persist(path: &Path, store: &LicenseStore) -> AppResult<()> {
    let json = serde_json::to_vec(store).map_err(|error| {
        AppError::new(
            "LICENSE_STATE_WRITE_FAILED",
            format!("Could not encode local license state: {error}"),
        )
    })?;
    let protected = protected_key::protect_license_data(&json)?;
    let temporary_path = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let write_result = (|| -> AppResult<()> {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)?;
        file.write_all(&protected)?;
        file.sync_all()?;
        replace_state_file(&temporary_path, path)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(temporary_path);
    }
    write_result
}

#[cfg(windows)]
fn replace_state_file(source: &Path, destination: &Path) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };

    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let succeeded = unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if succeeded == 0 {
        return Err(AppError::from(std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(not(windows))]
fn replace_state_file(source: &Path, destination: &Path) -> AppResult<()> {
    fs::rename(source, destination)?;
    Ok(())
}

fn parse_date(value: &str) -> AppResult<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| AppError::validation("License dates must use YYYY-MM-DD format"))
}

fn format_date(value: NaiveDate) -> String {
    value.format("%Y-%m-%d").to_string()
}

fn decode_hex(value: &str) -> AppResult<Vec<u8>> {
    let value = value.trim();
    if value.len() % 2 != 0 {
        return Err(AppError::new(
            "LICENSE_CONFIG_INVALID",
            "The application license verification key is malformed",
        ));
    }

    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16).map_err(|error| {
                AppError::new(
                    "LICENSE_CONFIG_INVALID",
                    format!("The application license verification key is malformed: {error}"),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::{Signer, SigningKey};
    use rand_core::OsRng;

    fn signed_key(signing_key: &SigningKey, payload: &LicensePayload) -> String {
        let payload = serde_json::to_vec(payload).unwrap();
        let signature = signing_key.sign(&payload);
        format!(
            "sfs1.{}.{}",
            URL_SAFE_NO_PAD.encode(payload),
            URL_SAFE_NO_PAD.encode(signature.to_bytes())
        )
    }

    #[test]
    fn verification_accepts_a_valid_signature_and_rejects_tampering() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let payload = LicensePayload {
            version: 1,
            product: PRODUCT_ID.to_string(),
            activation_id: uuid::Uuid::new_v4().to_string(),
            school: "Example School".to_string(),
            starts_on: "2026-10-27".to_string(),
            expires_on: "2027-10-26".to_string(),
        };
        let key = signed_key(&signing_key, &payload);
        assert!(verify_license_with_key(&key, &signing_key.verifying_key().to_bytes()).is_ok());

        let mut tampered = key.clone();
        let final_character = tampered.pop().unwrap();
        tampered.push(if final_character == 'A' { 'B' } else { 'A' });
        assert!(
            verify_license_with_key(&tampered, &signing_key.verifying_key().to_bytes()).is_err()
        );
    }

    #[test]
    fn trial_transitions_to_five_day_grace_then_read_only() {
        let store = LicenseStore {
            activation_id: uuid::Uuid::new_v4().to_string(),
            trial_started_on: "2026-10-01".to_string(),
            highest_seen_on: "2026-10-01".to_string(),
            license_key: None,
        };
        let grace = status_for(&store, parse_date("2026-10-15").unwrap()).unwrap();
        assert_eq!(grace.state, "grace");
        assert_eq!(grace.grace_days_remaining, 5);
        assert_eq!(grace.trial_started_on, "2026-10-01");
        assert_eq!(grace.trial_ends_on, "2026-10-15");

        let read_only = status_for(&store, parse_date("2026-10-20").unwrap()).unwrap();
        assert_eq!(read_only.state, "readOnly");
    }
}
