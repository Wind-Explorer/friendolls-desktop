use crate::get_app_handle;
use crate::init::lifecycle::construct_user_session;
use crate::services::scene::close_splash_window;
use crate::services::welcome::close_welcome_window;
use crate::state::auth::get_auth_pass_with_refresh;
use crate::{lock_r, lock_w, state::FDOLL};
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

#[derive(Debug, Deserialize)]
struct LoginResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
struct RegisterResponse {
    id: String,
}

#[derive(Debug, Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Debug, Serialize)]
struct RegisterRequest<'a> {
    email: &'a str,
    password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct ChangePasswordRequest<'a> {
    #[serde(rename = "currentPassword")]
    current_password: &'a str,
    #[serde(rename = "newPassword")]
    new_password: &'a str,
}

#[derive(Debug, Serialize)]
struct ResetPasswordRequest<'a> {
    #[serde(rename = "oldPassword")]
    old_password: &'a str,
    #[serde(rename = "newPassword")]
    new_password: &'a str,
}

fn build_auth_pass(access_token: String, expires_in: u64) -> Result<AuthPass, AuthError> {
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

pub async fn get_session_token() -> Option<AuthPass> {
    get_auth_pass_with_refresh().await
}

pub async fn get_access_token() -> Option<String> {
    get_session_token().await.map(|pass| pass.access_token)
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
            .map(|chunk| std::str::from_utf8(chunk).unwrap())
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
        Err(_e) => {
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

pub fn logout() -> Result<(), AuthError> {
    info!("Logging out user");
    lock_w!(FDOLL).auth.auth_pass = None;
    clear_auth_pass()?;
    Ok(())
}

pub async fn logout_and_restart() -> Result<(), AuthError> {
    logout()?;
    let app_handle = get_app_handle();
    app_handle.restart();
}

pub async fn with_auth(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(token) = get_access_token().await {
        request.header("Authorization", format!("Bearer {}", token))
    } else {
        request
    }
}

pub async fn login(email: &str, password: &str) -> Result<AuthPass, AuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        let clients = guard.network.clients.as_ref();
        if clients.is_none() {
            error!("Clients not initialized yet!");
            return Err(AuthError::InvalidConfig);
        }
        (
            guard.app_config.clone(),
            clients.unwrap().http_client.clone(),
        )
    };

    let base_url = app_config
        .api_base_url
        .as_ref()
        .ok_or(AuthError::InvalidConfig)?;
    let url = format!("{}/auth/login", base_url);

    let response = http_client
        .post(url)
        .json(&LoginRequest { email, password })
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::RequestFailed(format!(
            "Status: {}, Body: {}",
            status, error_text
        )));
    }

    let login_response: LoginResponse = response.json().await?;
    let auth_pass = build_auth_pass(login_response.access_token, login_response.expires_in)?;
    lock_w!(FDOLL).auth.auth_pass = Some(auth_pass.clone());
    save_auth_pass(&auth_pass)?;
    Ok(auth_pass)
}

pub async fn register(
    email: &str,
    password: &str,
    name: Option<&str>,
    username: Option<&str>,
) -> Result<String, AuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        let clients = guard.network.clients.as_ref();
        if clients.is_none() {
            error!("Clients not initialized yet!");
            return Err(AuthError::InvalidConfig);
        }
        (
            guard.app_config.clone(),
            clients.unwrap().http_client.clone(),
        )
    };

    let base_url = app_config
        .api_base_url
        .as_ref()
        .ok_or(AuthError::InvalidConfig)?;
    let url = format!("{}/auth/register", base_url);

    let response = http_client
        .post(url)
        .json(&RegisterRequest {
            email,
            password,
            name,
            username,
        })
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::RequestFailed(format!(
            "Status: {}, Body: {}",
            status, error_text
        )));
    }

    let register_response: RegisterResponse = response.json().await?;
    Ok(register_response.id)
}

pub async fn change_password(
    current_password: &str,
    new_password: &str,
) -> Result<(), AuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        let clients = guard.network.clients.as_ref();
        if clients.is_none() {
            error!("Clients not initialized yet!");
            return Err(AuthError::InvalidConfig);
        }
        (
            guard.app_config.clone(),
            clients.unwrap().http_client.clone(),
        )
    };

    let base_url = app_config
        .api_base_url
        .as_ref()
        .ok_or(AuthError::InvalidConfig)?;
    let url = format!("{}/auth/change-password", base_url);

    let response = with_auth(
        http_client.post(url).json(&ChangePasswordRequest {
            current_password,
            new_password,
        }),
    )
    .await
    .send()
    .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::RequestFailed(format!(
            "Status: {}, Body: {}",
            status, error_text
        )));
    }

    Ok(())
}

pub async fn reset_password(old_password: &str, new_password: &str) -> Result<(), AuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        let clients = guard.network.clients.as_ref();
        if clients.is_none() {
            error!("Clients not initialized yet!");
            return Err(AuthError::InvalidConfig);
        }
        (
            guard.app_config.clone(),
            clients.unwrap().http_client.clone(),
        )
    };

    let base_url = app_config
        .api_base_url
        .as_ref()
        .ok_or(AuthError::InvalidConfig)?;
    let url = format!("{}/auth/reset-password", base_url);

    let response = with_auth(
        http_client.post(url).json(&ResetPasswordRequest {
            old_password,
            new_password,
        }),
    )
    .await
    .send()
    .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(AuthError::RequestFailed(format!(
            "Status: {}, Body: {}",
            status, error_text
        )));
    }

    Ok(())
}

pub async fn refresh_token(access_token: &str) -> Result<AuthPass, AuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        (
            guard.app_config.clone(),
            guard
                .network
                .clients
                .as_ref()
                .expect("clients present")
                .http_client
                .clone(),
        )
    };

    let base_url = app_config
        .api_base_url
        .as_ref()
        .ok_or(AuthError::InvalidConfig)?;
    let url = format!("{}/auth/refresh", base_url);

    let response = http_client
        .post(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        error!("Token refresh failed with status {}: {}", status, error_text);
        return Err(AuthError::RefreshFailed);
    }

    let refresh_response: LoginResponse = response.json().await?;
    let auth_pass = build_auth_pass(refresh_response.access_token, refresh_response.expires_in)?;
    lock_w!(FDOLL).auth.auth_pass = Some(auth_pass.clone());
    save_auth_pass(&auth_pass)?;
    Ok(auth_pass)
}

pub async fn login_and_init_session(email: &str, password: &str) -> Result<(), AuthError> {
    login(email, password).await?;
    close_welcome_window();
    tauri::async_runtime::spawn(async {
        construct_user_session().await;
        close_splash_window();
    });
    Ok(())
}
