use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{lock_r, lock_w, state::FDOLL};

use super::storage::{build_auth_pass, save_auth_pass, AuthError, AuthPass};

#[derive(Debug, Deserialize)]
pub struct StartSsoResponse {
    pub state: String,
    #[serde(rename = "authorizeUrl")]
    pub authorize_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresIn")]
    expires_in: u64,
    #[serde(rename = "refreshToken")]
    refresh_token: String,
    #[serde(rename = "refreshExpiresIn")]
    refresh_expires_in: u64,
}

#[derive(Debug, Serialize)]
struct StartSsoRequest<'a> {
    provider: &'a str,
    #[serde(rename = "redirectUri")]
    redirect_uri: &'a str,
}

#[derive(Debug, Serialize)]
struct ExchangeSsoCodeRequest<'a> {
    code: &'a str,
}

#[derive(Debug, Serialize)]
struct RefreshTokenRequest<'a> {
    #[serde(rename = "refreshToken")]
    refresh_token: &'a str,
}

#[derive(Debug, Serialize)]
struct LogoutRequest<'a> {
    #[serde(rename = "refreshToken")]
    refresh_token: &'a str,
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

pub async fn start_sso(provider: &str, redirect_uri: &str) -> Result<StartSsoResponse, AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/sso/start", base_url))
        .json(&StartSsoRequest {
            provider,
            redirect_uri,
        })
        .send()
        .await?;

    ensure_success(response).await?.json().await.map_err(AuthError::from)
}

pub async fn exchange_sso_code(code: &str) -> Result<AuthPass, AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/sso/exchange", base_url))
        .json(&ExchangeSsoCodeRequest { code })
        .send()
        .await?;

    let token_response: TokenResponse = ensure_success(response).await?.json().await?;
    build_auth_pass(
        token_response.access_token,
        token_response.expires_in,
        token_response.refresh_token,
        token_response.refresh_expires_in,
    )
}

pub async fn refresh_token(refresh_token: &str) -> Result<AuthPass, AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/refresh", base_url))
        .json(&RefreshTokenRequest { refresh_token })
        .send()
        .await?;

    let token_response: TokenResponse = ensure_success(response).await?.json().await?;
    let auth_pass = build_auth_pass(
        token_response.access_token,
        token_response.expires_in,
        token_response.refresh_token,
        token_response.refresh_expires_in,
    )?;

    lock_w!(FDOLL).auth.auth_pass = Some(auth_pass.clone());
    save_auth_pass(&auth_pass)?;
    Ok(auth_pass)
}

pub async fn logout_remote(refresh_token: &str) -> Result<(), AuthError> {
    let (base_url, http_client) = auth_http_context()?;
    let response = http_client
        .post(format!("{}/auth/logout", base_url))
        .json(&LogoutRequest { refresh_token })
        .send()
        .await?;

    ensure_success(response).await?;
    Ok(())
}

pub fn persist_auth_pass(auth_pass: &AuthPass) -> Result<(), AuthError> {
    lock_w!(FDOLL).auth.auth_pass = Some(auth_pass.clone());
    save_auth_pass(auth_pass)?;
    Ok(())
}
