use crate::{core::state::FDOLL, lock_r, lock_w, APP_HANDLE};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use keyring::Entry;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri_plugin_opener::OpenerExt;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tracing::{error, info, warn};
use url::form_urlencoded;

static REFRESH_LOCK: once_cell::sync::Lazy<Mutex<()>> =
    once_cell::sync::Lazy::new(|| Mutex::new(()));

static AUTH_SUCCESS_HTML: &str = include_str!("../../assets/auth-success.html");
const SERVICE_NAME: &str = "friendolls";

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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
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
    let json = serde_json::to_string(auth_pass)?;
    info!("Original JSON length: {}", json.len());
    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder
        .write_all(json.as_bytes())
        .map_err(|e| OAuthError::SerializationError(serde_json::Error::io(e)))?;
    let compressed = encoder
        .finish()
        .map_err(|e| OAuthError::SerializationError(serde_json::Error::io(e)))?;
    info!("Compressed length: {}", compressed.len());
    let encoded = URL_SAFE_NO_PAD.encode(&compressed);
    info!("Encoded length: {}", encoded.len());

    // Windows keyring has a 2560-byte UTF-16 limit, which means 1280 chars max
    // Split into chunks of 1200 chars to be safe
    const CHUNK_SIZE: usize = 1200;
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(CHUNK_SIZE)
        .map(|chunk| std::str::from_utf8(chunk).unwrap())
        .collect();

    info!("Splitting auth pass into {} chunks", chunks.len());

    // Save chunk count
    let count_entry = Entry::new(SERVICE_NAME, "auth_pass_count")?;
    count_entry.set_password(&chunks.len().to_string())?;

    // Save each chunk
    for (i, chunk) in chunks.iter().enumerate() {
        let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
        entry.set_password(chunk)?;
    }

    info!(
        "Auth pass saved to keyring successfully in {} chunks",
        chunks.len()
    );
    Ok(())
}

/// Load auth_pass from secure storage (keyring).
pub fn load_auth_pass() -> Result<Option<AuthPass>, OAuthError> {
    info!("Reading credentials from keyring");

    // Get chunk count
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
            return Err(OAuthError::KeyringError(e));
        }
    };

    info!("Loading {} auth pass chunks from keyring", chunk_count);

    // Reassemble chunks
    let mut encoded = String::new();
    for i in 0..chunk_count {
        let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
        match entry.get_password() {
            Ok(chunk) => encoded.push_str(&chunk),
            Err(e) => {
                error!("Failed to load chunk {} from keyring", i);
                return Err(OAuthError::KeyringError(e));
            }
        }
    }

    info!("Reassembled encoded length: {}", encoded.len());

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
        Ok(v) => {
            info!("Deserialized auth pass from keyring");
            v
        }
        Err(_e) => {
            error!("Failed to decode auth pass from keyring");
            return Ok(None);
        }
    };

    info!("Auth pass loaded from keyring");
    Ok(Some(auth_pass))
}

/// Clear auth_pass from secure storage and app state.
pub fn clear_auth_pass() -> Result<(), OAuthError> {
    // Try to get chunk count
    let count_entry = Entry::new(SERVICE_NAME, "auth_pass_count")?;
    let chunk_count = match count_entry.get_password() {
        Ok(count_str) => count_str.parse::<usize>().unwrap_or(0),
        Err(_) => 0,
    };

    // Delete all chunks
    for i in 0..chunk_count {
        let entry = Entry::new(SERVICE_NAME, &format!("auth_pass_{}", i))?;
        let _ = entry.delete_credential();
    }

    // Delete chunk count
    let _ = count_entry.delete_credential();

    info!("Auth pass cleared from keyring successfully");
    Ok(())
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

    // Bind the server FIRST to ensure port is open
    // We bind synchronously using std::net::TcpListener then convert to tokio::net::TcpListener
    // to ensure the port is bound before we open the browser.
    let std_listener = match std::net::TcpListener::bind(&app_config.auth.redirect_host) {
        Ok(s) => {
            s.set_nonblocking(true).unwrap();
            s
        }
        Err(e) => {
            error!("Failed to bind callback server: {}", e);
            return;
        }
    };

    info!(
        "Listening on {} for /callback",
        &app_config.auth.redirect_host
    );

    tauri::async_runtime::spawn(async move {
        let listener = match TcpListener::from_std(std_listener) {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to create async listener: {}", e);
                return;
            }
        };

        match listen_for_callback(listener).await {
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
/// - Required callback parameters are missing
/// - Timeout is reached before callback is received
async fn listen_for_callback(listener: TcpListener) -> Result<OAuthCallbackParams, OAuthError> {
    // Set a 5-minute timeout
    let timeout = Duration::from_secs(300);
    let start_time = Instant::now();

    loop {
        let elapsed = start_time.elapsed();
        if elapsed > timeout {
            warn!("Callback listener timed out after 5 minutes");
            return Err(OAuthError::CallbackTimeout);
        }

        let accept_result = tokio::time::timeout(timeout - elapsed, listener.accept()).await;

        let (mut stream, _) = match accept_result {
            Ok(Ok(res)) => res,
            Ok(Err(e)) => {
                warn!("Accept error: {}", e);
                continue;
            }
            Err(_) => {
                warn!("Callback listener timed out after 5 minutes");
                return Err(OAuthError::CallbackTimeout);
            }
        };

        let mut buffer = [0; 4096];
        let n = match stream.read(&mut buffer).await {
            Ok(n) if n > 0 => n,
            _ => continue,
        };

        let request = String::from_utf8_lossy(&buffer[..n]);
        let first_line = request.lines().next().unwrap_or("");
        let mut parts = first_line.split_whitespace();

        match (parts.next(), parts.next()) {
            (Some("GET"), Some(path)) if path.starts_with("/callback") => {
                let full_url = format!("http://localhost{}", path);
                let url = match url::Url::parse(&full_url) {
                    Ok(u) => u,
                    Err(_) => continue,
                };

                let params: std::collections::HashMap<_, _> =
                    url.query_pairs().into_owned().collect();

                info!("Received OAuth callback");

                let find_param = |key: &str| -> Result<String, OAuthError> {
                    params
                        .get(key)
                        .cloned()
                        .ok_or_else(|| OAuthError::MissingParameter(key.to_string()))
                };

                let callback_params = OAuthCallbackParams {
                    state: find_param("state")?,
                    session_state: find_param("session_state")?,
                    iss: find_param("iss")?,
                    code: find_param("code")?,
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                    AUTH_SUCCESS_HTML.len(),
                    AUTH_SUCCESS_HTML
                );

                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.flush().await;

                info!("Callback processed, stopping listener");
                return Ok(callback_params);
            }
            (Some("GET"), Some("/health")) => {
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK";
                let _ = stream.write_all(response.as_bytes()).await;
            }
            _ => {
                let response = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                let _ = stream.write_all(response.as_bytes()).await;
            }
        }
    }
}
