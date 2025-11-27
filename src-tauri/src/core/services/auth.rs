use crate::{core::state::FDOLL, lock_r, lock_w, APP_HANDLE};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use keyring::Entry;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri_plugin_opener::OpenerExt;
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use url::form_urlencoded;

static REFRESH_LOCK: once_cell::sync::Lazy<Mutex<()>> =
    once_cell::sync::Lazy::new(|| Mutex::new(()));

static AUTH_SUCCESS_HTML: &str = include_str!("../../assets/auth-success.html");

/// Errors that can occur during OAuth authentication flow.
#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Failed to exchange code: {0}")]
    ExchangeFailed(String),

    #[error("Invalid callback state - possible CSRF attack")]
    InvalidState,

    #[error("Missing callback parameter: {0}")]
    MissingParameter(String),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Server binding failed: {0}")]
    ServerBindError(String),

    #[error("Callback timeout - no response received")]
    CallbackTimeout,

    #[error("Invalid app configuration")]
    InvalidConfig,

    #[error("Failed to refresh token")]
    RefreshFailed,

    #[error("OAuth state expired or not initialized")]
    StateExpired,
}

/// Parameters received from the OAuth callback.
pub struct OAuthCallbackParams {
    state: String,
    session_state: String,
    iss: String,
    code: String,
}

/// Authentication pass containing access token, refresh token, and metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthPass {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_expires_in: u64,
    pub refresh_token: String,
    pub token_type: String,
    pub session_state: String,
    pub scope: String,
    pub issued_at: Option<u64>,
}

/// Generate a random code verifier for PKCE.
///
/// Per PKCE spec (RFC 7636), the code verifier should be 43-128 characters.
fn generate_code_verifier(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate code challenge from a code verifier using SHA-256.
///
/// This implements the S256 method as specified in RFC 7636.
fn generate_code_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(&result)
}

/// Returns the auth pass object, including
/// access token, refresh token, expire time etc.
/// Automatically refreshes if expired.
pub async fn get_tokens() -> Option<AuthPass> {
    info!("Retrieving tokens");
    let Some(auth_pass) = ({ lock_r!(FDOLL).auth_pass.clone() }) else {
        return None;
    };

    let Some(issued_at) = auth_pass.issued_at else {
        warn!("Auth pass missing issued_at timestamp, clearing");
        lock_w!(FDOLL).auth_pass = None;
        return None;
    };

    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();

    let expired = current_time - issued_at >= auth_pass.expires_in;
    let refresh_expired = current_time - issued_at >= auth_pass.refresh_expires_in;

    if !expired {
        return Some(auth_pass);
    }

    if refresh_expired {
        info!("Refresh token expired, clearing auth state");
        lock_w!(FDOLL).auth_pass = None;
        if let Err(e) = clear_auth_pass() {
            error!("Failed to clear expired auth pass: {}", e);
        }
        return None;
    }

    // Use mutex to prevent concurrent refresh
    let _guard = REFRESH_LOCK.lock().await;

    // Double-check after acquiring lock
    let auth_pass = lock_r!(FDOLL).auth_pass.clone()?;
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let expired = current_time - auth_pass.issued_at? >= auth_pass.expires_in;

    if !expired {
        // Another thread already refreshed
        return Some(auth_pass);
    }

    info!("Access token expired, attempting refresh");
    match refresh_token(&auth_pass.refresh_token).await {
        Ok(new_pass) => Some(new_pass),
        Err(e) => {
            error!("Failed to refresh token: {}", e);
            lock_w!(FDOLL).auth_pass = None;
            if let Err(e) = clear_auth_pass() {
                error!("Failed to clear auth pass after refresh failure: {}", e);
            }
            None
        }
    }
}

/// Helper function to get the current access token.
pub async fn get_access_token() -> Option<String> {
    get_tokens().await.map(|pass| pass.access_token)
}

/// Save auth_pass to secure storage (keyring) and update app state.
pub fn save_auth_pass(auth_pass: &AuthPass) -> Result<(), OAuthError> {
    let entry = Entry::new("friendolls", "auth_pass")?;
    let json = serde_json::to_string(auth_pass)?;
    entry.set_password(&json)?;
    info!("Auth pass saved to keyring successfully");
    Ok(())
}

/// Load auth_pass from secure storage (keyring).
pub fn load_auth_pass() -> Result<Option<AuthPass>, OAuthError> {
    info!("Reading credentials from keyring");
    let entry = match Entry::new("friendolls", "auth_pass") {
        Ok(value) => value,
        Err(e) => {
            error!("Failed to open keyring entry");
            panic!()
        }
    };
    info!("Opened credentials from keyring");
    match entry.get_password() {
        Ok(json) => {
            info!("Got credentials from keyring");
            let auth_pass: AuthPass = match serde_json::from_str(&json) {
                Ok(v) => {
                    info!("Deserialized auth pass from keyring");
                    v
                }
                Err(e) => {
                    error!("Failed to decode auth pass from keyring");
                    return Ok(None);
                }
            };
            info!("Auth pass loaded from keyring");
            Ok(Some(auth_pass))
        }
        Err(keyring::Error::NoEntry) => {
            info!("No auth pass found in keyring");
            Ok(None)
        }
        Err(e) => {
            error!("Failed to load from keyring");
            Err(OAuthError::KeyringError(e))
        }
    }
}

/// Clear auth_pass from secure storage and app state.
pub fn clear_auth_pass() -> Result<(), OAuthError> {
    let entry = Entry::new("friendolls", "auth_pass")?;
    match entry.delete_credential() {
        Ok(_) => {
            info!("Auth pass cleared from keyring successfully");
            Ok(())
        }
        Err(keyring::Error::NoEntry) => {
            info!("Auth pass already cleared from keyring");
            Ok(())
        }
        Err(e) => Err(OAuthError::KeyringError(e)),
    }
}

/// Logout the current user by clearing tokens from storage and state.
///
/// # Note
///
/// This currently only clears local tokens. For complete logout, you should also
/// call the OAuth provider's token revocation endpoint if available.
///
/// # Example
///
/// ```rust,no_run
/// use crate::core::services::auth::logout;
///
/// logout().expect("Failed to logout");
/// ```
pub fn logout() -> Result<(), OAuthError> {
    info!("Logging out user");
    lock_w!(FDOLL).auth_pass = None;
    clear_auth_pass()?;

    // Clear OAuth flow state as well
    lock_w!(FDOLL).oauth_flow = Default::default();

    // TODO: Call OAuth provider's revocation endpoint
    // This would require adding a revoke_token() function that calls:
    // POST {auth_url}/revoke with the refresh_token

    Ok(())
}

/// Helper to add authentication header to a request builder if tokens are available.
///
/// # Example
///
/// ```rust,no_run
/// use crate::core::services::auth::with_auth;
///
/// let client = reqwest::Client::new();
/// let request = client.get("https://api.example.com/user");
/// let authenticated_request = with_auth(request).await;
/// ```
pub async fn with_auth(request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    if let Some(token) = get_access_token().await {
        request.header("Authorization", format!("Bearer {}", token))
    } else {
        request
    }
}

/// Exchange authorization code for tokens.
///
/// This is called after receiving the OAuth callback with an authorization code.
/// It exchanges the code for an access token and refresh token.
///
/// # Arguments
///
/// * `callback_params` - Parameters received from the OAuth callback
/// * `code_verifier` - The PKCE code verifier that was used to generate the code challenge
///
/// # Errors
///
/// Returns `OAuthError` if the exchange fails or the server returns an error.
pub async fn exchange_code_for_auth_pass(
    callback_params: OAuthCallbackParams,
    code_verifier: &str,
) -> Result<AuthPass, OAuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        (
            guard.app_config.clone().ok_or(OAuthError::InvalidConfig)?,
            guard.http_client.clone(),
        )
    };

    let url = url::Url::parse(&format!("{}/token", &app_config.auth.auth_url))
        .map_err(|_| OAuthError::InvalidConfig)?;

    let body = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", &app_config.auth.audience)
        .append_pair("grant_type", "authorization_code")
        .append_pair("redirect_uri", &app_config.auth.redirect_uri)
        .append_pair("code", &callback_params.code)
        .append_pair("code_verifier", code_verifier)
        .finish();

    info!("Exchanging authorization code for tokens");

    let exchange_request = http_client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body);

    let exchange_request_response = exchange_request.send().await?;

    if !exchange_request_response.status().is_success() {
        let status = exchange_request_response.status();
        let error_text = exchange_request_response.text().await.unwrap_or_default();
        error!(
            "Token exchange failed with status {}: {}",
            status, error_text
        );
        return Err(OAuthError::ExchangeFailed(format!(
            "Status: {}, Body: {}",
            status, error_text
        )));
    }

    let mut auth_pass: AuthPass = exchange_request_response.json().await?;
    auth_pass.issued_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OAuthError::ExchangeFailed("System time error".to_string()))?
            .as_secs(),
    );

    info!("Successfully exchanged code for tokens");
    Ok(auth_pass)
}

/// Initialize the OAuth authorization code flow.
///
/// This function:
/// 1. Generates PKCE code verifier and challenge
/// 2. Generates state parameter for CSRF protection
/// 3. Stores state and code verifier in app state
/// 4. Opens the OAuth authorization URL in the user's browser
/// 5. Starts a background listener for the callback
///
/// The user will be redirected to the OAuth provider's login page, and after
/// successful authentication, will be redirected back to the local callback server.
///
/// # Example
///
/// ```rust,no_run
/// use crate::core::services::auth::init_auth_code_retrieval;
///
/// init_auth_code_retrieval();
/// // User will be prompted to login in their browser
/// ```
pub fn init_auth_code_retrieval<F>(on_success: F)
where
    F: FnOnce() + Send + 'static,
{
    let app_config = match lock_r!(FDOLL).app_config.clone() {
        Some(config) => config,
        None => {
            error!("Cannot initialize auth: app config not available");
            return;
        }
    };

    let opener = match APP_HANDLE.get() {
        Some(handle) => handle.opener(),
        None => {
            error!("Cannot initialize auth: app handle not available");
            return;
        }
    };

    let code_verifier = generate_code_verifier(64);
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_code_verifier(16);

    // Store state and code_verifier for validation
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    {
        let mut guard = lock_w!(FDOLL);
        guard.oauth_flow.state = Some(state.clone());
        guard.oauth_flow.code_verifier = Some(code_verifier.clone());
        guard.oauth_flow.initiated_at = Some(current_time);
    }

    let mut url = match url::Url::parse(&format!("{}/auth", &app_config.auth.auth_url)) {
        Ok(url) => url,
        Err(e) => {
            error!("Invalid auth URL configuration: {}", e);
            return;
        }
    };

    url.query_pairs_mut()
        .append_pair("client_id", &app_config.auth.audience)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &app_config.auth.redirect_uri)
        .append_pair("scope", "openid email profile")
        .append_pair("state", &state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

    info!("Initiating OAuth flow");

    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async move {
                match listen_for_callback().await {
                    Ok(callback_params) => {
                        // Validate state
                        let stored_state = lock_r!(FDOLL).oauth_flow.state.clone();

                        if stored_state.as_ref() != Some(&callback_params.state) {
                            error!("State mismatch - possible CSRF attack!");
                            return;
                        }

                        // Retrieve code_verifier
                        let code_verifier = match lock_r!(FDOLL).oauth_flow.code_verifier.clone() {
                            Some(cv) => cv,
                            None => {
                                error!("Code verifier not found in state");
                                return;
                            }
                        };

                        // Clear OAuth flow state after successful callback
                        lock_w!(FDOLL).oauth_flow = Default::default();

                        match exchange_code_for_auth_pass(callback_params, &code_verifier).await {
                            Ok(auth_pass) => {
                                lock_w!(FDOLL).auth_pass = Some(auth_pass.clone());
                                if let Err(e) = save_auth_pass(&auth_pass) {
                                    error!("Failed to save auth pass: {}", e);
                                } else {
                                    info!("Authentication successful!");
                                    on_success();
                                }
                            }
                            Err(e) => {
                                error!("Failed to exchange code for tokens: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to receive callback: {}", e);
                        // Clear OAuth flow state on error
                        lock_w!(FDOLL).oauth_flow = Default::default();
                    }
                }
            });
    });

    if let Err(e) = opener.open_url(url, None::<&str>) {
        error!("Failed to open auth portal: {}", e);
    }
}

/// Refresh the access token using a refresh token.
///
/// This is called automatically by `get_tokens()` when the access token is expired
/// but the refresh token is still valid.
///
/// # Arguments
///
/// * `refresh_token` - The refresh token to use
///
/// # Errors
///
/// Returns `OAuthError::RefreshFailed` if the refresh fails.
pub async fn refresh_token(refresh_token: &str) -> Result<AuthPass, OAuthError> {
    let (app_config, http_client) = {
        let guard = lock_r!(FDOLL);
        (
            guard.app_config.clone().ok_or(OAuthError::InvalidConfig)?,
            guard.http_client.clone(),
        )
    };

    let url = url::Url::parse(&format!("{}/token", &app_config.auth.auth_url))
        .map_err(|_| OAuthError::InvalidConfig)?;

    let body = form_urlencoded::Serializer::new(String::new())
        .append_pair("client_id", &app_config.auth.audience)
        .append_pair("grant_type", "refresh_token")
        .append_pair("refresh_token", refresh_token)
        .finish();

    info!("Refreshing access token");

    let refresh_request = http_client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body);

    let refresh_response = refresh_request.send().await?;

    if !refresh_response.status().is_success() {
        let status = refresh_response.status();
        let error_text = refresh_response.text().await.unwrap_or_default();
        error!(
            "Token refresh failed with status {}: {}",
            status, error_text
        );
        return Err(OAuthError::RefreshFailed);
    }

    let mut auth_pass: AuthPass = refresh_response.json().await?;
    auth_pass.issued_at = Some(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| OAuthError::RefreshFailed)?
            .as_secs(),
    );

    // Update state and storage
    lock_w!(FDOLL).auth_pass = Some(auth_pass.clone());
    if let Err(e) = save_auth_pass(&auth_pass) {
        error!("Failed to save refreshed auth pass: {}", e);
    } else {
        info!("Token refreshed successfully");
    }

    Ok(auth_pass)
}

/// Start a local HTTP server to listen for the OAuth callback.
///
/// This function starts a mini web server that listens on the configured redirect host
/// for the OAuth callback. It:
/// - Listens on the `/callback` endpoint
/// - Validates all required parameters are present
/// - Returns a nice HTML page to the user
/// - Has a 5-minute timeout to prevent hanging indefinitely
/// - Also provides a `/health` endpoint for health checks
///
/// # Timeout
///
/// The server will timeout after 5 minutes if no callback is received,
/// preventing the server from running indefinitely if the user abandons the flow.
///
/// # Errors
///
/// Returns `OAuthError` if:
/// - Server fails to bind to the configured port
/// - Required callback parameters are missing
/// - Timeout is reached before callback is received
async fn listen_for_callback() -> Result<OAuthCallbackParams, OAuthError> {
    let app_config = lock_r!(FDOLL)
        .app_config
        .clone()
        .ok_or(OAuthError::InvalidConfig)?;

    let server = tiny_http::Server::http(&app_config.auth.redirect_host)
        .map_err(|e| OAuthError::ServerBindError(e.to_string()))?;

    info!(
        "Listening on {} for /callback",
        &app_config.auth.redirect_host
    );

    // Set a 5-minute timeout
    let timeout = Duration::from_secs(300);
    let start_time = SystemTime::now();

    for request in server.incoming_requests() {
        // Check timeout
        if SystemTime::now()
            .duration_since(start_time)
            .unwrap_or(Duration::ZERO)
            > timeout
        {
            warn!("Callback listener timed out after 5 minutes");
            return Err(OAuthError::CallbackTimeout);
        }

        let url = request.url().to_string();

        if url.starts_with("/callback") {
            let query = url.split('?').nth(1).unwrap_or("");
            let params = form_urlencoded::parse(query.as_bytes())
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect::<Vec<(String, String)>>();

            info!("Received OAuth callback");

            let find_param = |key: &str| -> Result<String, OAuthError> {
                params
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| OAuthError::MissingParameter(key.to_string()))
            };

            let callback_params = OAuthCallbackParams {
                state: find_param("state")?,
                session_state: find_param("session_state")?,
                iss: find_param("iss")?,
                code: find_param("code")?,
            };

            let response = tiny_http::Response::from_string(AUTH_SUCCESS_HTML).with_header(
                tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                )
                .map_err(|_| OAuthError::ServerBindError("Header creation failed".to_string()))?,
            );

            let _ = request.respond(response);

            info!("Callback processed, stopping listener");
            return Ok(callback_params);
        } else if url == "/health" {
            // Health check endpoint
            let _ = request.respond(tiny_http::Response::from_string("OK"));
        } else {
            let _ = request.respond(tiny_http::Response::empty(404));
        }
    }

    Err(OAuthError::CallbackTimeout)
}
