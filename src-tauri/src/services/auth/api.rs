use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{lock_r, lock_w, state::FDOLL};

use super::storage::{build_auth_pass, save_auth_pass, AuthError, AuthPass};

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

fn auth_http_context() -> Result<(String, reqwest::Client), AuthError> {
    let guard = lock_r!(FDOLL);
    let clients = guard.network.clients.as_ref().ok_or_else(|| {
        error!("Clients not initialized yet!");
        AuthError::InvalidConfig
    })?;

    let base_url = guard
        .app_config
        .api_base_url
        .clone()
        .ok_or(AuthError::InvalidConfig)?;

    Ok((base_url, clients.http_client.clone()))
}

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response, AuthError> {
    if response.status().is_success() {
        return Ok(response);
    }

    let status = response.status();
    let error_text = response.text().await.unwrap_or_default();
    Err(AuthError::RequestFailed(format!(
        "Status: {}, Body: {}",
        status, error_text
    )))
}

pub async fn with_auth(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(token) = super::session::get_access_token().await {
        request.header("Authorization", format!("Bearer {}", token))
    } else {
        request
    }
}

pub async fn login(email: &str, password: &str) -> Result<AuthPass, AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/login", base_url))
        .json(&LoginRequest { email, password })
        .send()
        .await?;

    let login_response: LoginResponse = ensure_success(response).await?.json().await?;
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
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/register", base_url))
        .json(&RegisterRequest {
            email,
            password,
            name,
            username,
        })
        .send()
        .await?;

    let register_response: RegisterResponse = ensure_success(response).await?.json().await?;
    Ok(register_response.id)
}

pub async fn change_password(
    current_password: &str,
    new_password: &str,
) -> Result<(), AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = with_auth(http_client.post(format!("{}/auth/change-password", base_url)).json(
        &ChangePasswordRequest {
            current_password,
            new_password,
        },
    ))
    .await
    .send()
    .await?;

    ensure_success(response).await?;
    Ok(())
}

pub async fn reset_password(old_password: &str, new_password: &str) -> Result<(), AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = with_auth(http_client.post(format!("{}/auth/reset-password", base_url)).json(
        &ResetPasswordRequest {
            old_password,
            new_password,
        },
    ))
    .await
    .send()
    .await?;

    ensure_success(response).await?;
    Ok(())
}

pub async fn refresh_token(access_token: &str) -> Result<AuthPass, AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/refresh", base_url))
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
