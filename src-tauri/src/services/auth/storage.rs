use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::{error, info};

const SERVICE_NAME: &str = "friendolls";

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid app configuration")]
    InvalidConfig,

    #[error("Failed to refresh token")]
    RefreshFailed,

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPass {
    pub access_token: String,
    pub expires_in: u64,
    pub issued_at: Option<u64>,
}

pub(crate) fn build_auth_pass(
    access_token: String,
    expires_in: u64,
) -> Result<AuthPass, AuthError> {
    let issued_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| AuthError::RefreshFailed)?
        .as_secs();
    Ok(AuthPass {
        access_token,
        expires_in,
        issued_at: Some(issued_at),
    })
}

pub fn save_auth_pass(auth_pass: &AuthPass) -> Result<(), AuthError> {
    let json = serde_json::to_string(auth_pass)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(json.as_bytes())
        .map_err(|e| AuthError::SerializationError(serde_json::Error::io(e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| AuthError::SerializationError(serde_json::Error::io(e)))?;
    let encoded = URL_SAFE_NO_PAD.encode(&compressed);

    #[cfg(target_os = "windows")]
    {
        const CHUNK_SIZE: usize = 1200;
        let chunks: Vec<&str> = encoded
            .as_bytes()
            .chunks(CHUNK_SIZE)
            .map(|chunk| std::str::from_utf8(chunk).expect("base64 chunk is valid utf-8"))
            .collect();

        let count_entry = Entry::new(SERVICE_NAME, "auth_pass_count")?;
        count_entry.set_password(&chunks.len().to_string())?;

        for (i, chunk) in chunks.iter().enumerate() {
            let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
            entry.set_password(chunk)?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let entry = Entry::new(SERVICE_NAME, "auth_pass")?;
        entry.set_password(&encoded)?;
    }

    Ok(())
}

pub fn load_auth_pass() -> Result<Option<AuthPass>, AuthError> {
    #[cfg(target_os = "windows")]
    let encoded = {
        let count_entry = Entry::new(SERVICE_NAME, "auth_pass_count")?;
        let chunk_count = match count_entry.get_password() {
            Ok(count_str) => match count_str.parse::<usize>() {
                Ok(count) => count,
                Err(_) => {
                    error!("Invalid chunk count in keyring");
                    return Ok(None);
                }
            },
            Err(keyring::Error::NoEntry) => {
                info!("No auth pass found in keyring");
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to load chunk count from keyring");
                return Err(AuthError::KeyringError(e));
            }
        };

        let mut encoded = String::new();
        for i in 0..chunk_count {
            let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
            match entry.get_password() {
                Ok(chunk) => encoded.push_str(&chunk),
                Err(e) => {
                    error!("Failed to load chunk {} from keyring", i);
                    return Err(AuthError::KeyringError(e));
                }
            }
        }
        encoded
    };

    #[cfg(not(target_os = "windows"))]
    let encoded = {
        let entry = Entry::new(SERVICE_NAME, "auth_pass")?;
        match entry.get_password() {
            Ok(pass) => pass,
            Err(keyring::Error::NoEntry) => {
                info!("No auth pass found in keyring");
                return Ok(None);
            }
            Err(e) => {
                error!("Failed to load auth pass from keyring");
                return Err(AuthError::KeyringError(e));
            }
        }
    };

    let compressed = match URL_SAFE_NO_PAD.decode(&encoded) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to base64 decode auth pass from keyring: {}", e);
            return Ok(None);
        }
    };

    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut json = String::new();
    if let Err(e) = decoder.read_to_string(&mut json) {
        error!("Failed to decompress auth pass from keyring: {}", e);
        return Ok(None);
    }

    let auth_pass: AuthPass = match serde_json::from_str(&json) {
        Ok(v) => v,
        Err(_) => {
            error!("Failed to decode auth pass from keyring");
            return Ok(None);
        }
    };

    Ok(Some(auth_pass))
}

pub fn clear_auth_pass() -> Result<(), AuthError> {
    #[cfg(target_os = "windows")]
    {
        let count_entry = Entry::new(SERVICE_NAME, "auth_pass_count")?;
        let chunk_count = match count_entry.get_password() {
            Ok(count_str) => count_str.parse::<usize>().unwrap_or(0),
            Err(_) => 0,
        };

        for i in 0..chunk_count {
            let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
            let _ = entry.delete_credential();
        }

        let _ = count_entry.delete_credential();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let entry = Entry::new(SERVICE_NAME, "auth_pass")?;
        let _ = entry.delete_credential();
    }

    Ok(())
}
