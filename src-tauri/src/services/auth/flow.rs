use tauri_specta::Event as _;
use tauri_plugin_opener::OpenerExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::get_app_handle;
use crate::services::app_events::{AuthFlowStatus, AuthFlowUpdated, AuthFlowUpdatedPayload};
use crate::state::{begin_auth_flow, clear_auth_flow_state, is_auth_flow_active};
use crate::{lock_r, state::FDOLL};

use super::api::{exchange_sso_code, persist_auth_pass, start_sso};
use super::storage::AuthError;

static AUTH_SUCCESS_HTML: &str = include_str!("../../assets/auth-success.html");
static AUTH_CANCELLED_HTML: &str = include_str!("../../assets/auth-cancelled.html");
static AUTH_FAILED_HTML: &str = include_str!("../../assets/auth-failed.html");

pub struct OAuthCallbackParams {
    pub state: String,
    pub result: OAuthCallbackResult,
}

pub enum OAuthCallbackResult {
    Code(String),
    Error { message: String, cancelled: bool },
}

struct PendingOAuthCallback {
    stream: TcpStream,
    params: OAuthCallbackParams,
}

pub async fn start_browser_auth_flow(provider: &str) -> Result<(), AuthError> {
    let (flow_id, cancel_token) = begin_auth_flow();

    let bind_addr = "127.0.0.1:0";
    let std_listener = std::net::TcpListener::bind(bind_addr)
        .map_err(|e| AuthError::ServerBindError(e.to_string()))?;
    std_listener
        .set_nonblocking(true)
        .map_err(|e| AuthError::ServerBindError(e.to_string()))?;
    let local_addr = std_listener
        .local_addr()
        .map_err(|e| AuthError::ServerBindError(e.to_string()))?;

    let redirect_uri = format!("http://127.0.0.1:{}/callback", local_addr.port());
    let start_response = match start_sso(provider, &redirect_uri).await {
        Ok(response) => response,
        Err(err) => {
            clear_auth_flow_state(flow_id);
            return Err(err);
        }
    };

    let listener = TcpListener::from_std(std_listener)
        .map_err(|e| AuthError::ServerBindError(e.to_string()))?;
    let expected_state = start_response.state.clone();
    let auth_url = match start_response.authorize_url.clone() {
        Some(authorize_url) => authorize_url,
        None => match build_authorize_url(provider, &start_response.state) {
            Ok(url) => url,
            Err(err) => {
                clear_auth_flow_state(flow_id);
                return Err(err);
            }
        },
    };
    let provider_name = provider.to_string();

    if let Err(err) = get_app_handle().opener().open_url(auth_url, None::<&str>) {
        clear_auth_flow_state(flow_id);
        emit_auth_flow_event(
            provider,
            AuthFlowStatus::Failed,
            Some("Friendolls could not open your browser for sign-in.".to_string()),
        );
        return Err(err.into());
    }

    emit_auth_flow_event(provider, AuthFlowStatus::Started, None);
    tauri::async_runtime::spawn(async move {
        match listen_for_callback(listener, cancel_token.clone()).await {
            Ok(mut callback) => {
                if !is_auth_flow_active(flow_id) {
                    let _ = write_html_response(&mut callback.stream, AUTH_CANCELLED_HTML).await;
                    return;
                }

                if callback.params.state != expected_state {
                    error!("SSO state mismatch");
                    if let Err(err) = write_html_response(&mut callback.stream, AUTH_FAILED_HTML).await {
                        warn!("Failed to write auth failure response: {}", err);
                    }
                    emit_auth_flow_event(
                        &provider_name,
                        AuthFlowStatus::Failed,
                        Some("Sign-in verification failed. Please try again.".to_string()),
                    );
                    clear_auth_flow_state(flow_id);
                    return;
                }

                match callback.params.result {
                    OAuthCallbackResult::Code(code) => {
                        let auth_pass = match exchange_sso_code(&code).await {
                            Ok(auth_pass) => auth_pass,
                            Err(err) => {
                                error!("Failed to exchange SSO code: {}", err);
                                if let Err(write_err) =
                                    write_html_response(&mut callback.stream, AUTH_FAILED_HTML).await
                                {
                                    warn!("Failed to write auth failure response: {}", write_err);
                                }
                                emit_auth_flow_event(
                                    &provider_name,
                                    AuthFlowStatus::Failed,
                                    Some(
                                        "Friendolls could not complete sign-in. Please try again."
                                            .to_string(),
                                    ),
                                );
                                clear_auth_flow_state(flow_id);
                                return;
                            }
                        };

                        if !is_auth_flow_active(flow_id) {
                            let _ = write_html_response(&mut callback.stream, AUTH_CANCELLED_HTML).await;
                            return;
                        }

                        if let Err(err) = persist_auth_pass(&auth_pass) {
                            error!("Failed to persist SSO auth pass: {}", err);
                            if let Err(write_err) =
                                write_html_response(&mut callback.stream, AUTH_FAILED_HTML).await
                            {
                                warn!("Failed to write auth failure response: {}", write_err);
                            }
                            emit_auth_flow_event(
                                &provider_name,
                                AuthFlowStatus::Failed,
                                Some(
                                    "Friendolls could not complete sign-in. Please try again."
                                        .to_string(),
                                ),
                            );
                            clear_auth_flow_state(flow_id);
                            return;
                        }

                        if let Err(err) = super::session::finish_login_session().await {
                            error!("Failed to finalize desktop login session: {}", err);
                            if let Err(write_err) =
                                write_html_response(&mut callback.stream, AUTH_FAILED_HTML).await
                            {
                                warn!("Failed to write auth failure response: {}", write_err);
                            }
                            emit_auth_flow_event(
                                &provider_name,
                                AuthFlowStatus::Failed,
                                Some(
                                    "Signed in, but Friendolls could not open your session."
                                        .to_string(),
                                ),
                            );
                            clear_auth_flow_state(flow_id);
                        } else {
                            if let Err(err) =
                                write_html_response(&mut callback.stream, AUTH_SUCCESS_HTML).await
                            {
                                warn!("Failed to write auth success response: {}", err);
                            }
                            emit_auth_flow_event(&provider_name, AuthFlowStatus::Succeeded, None);
                            clear_auth_flow_state(flow_id);
                        }
                    }
                    OAuthCallbackResult::Error { message, cancelled } => {
                        let response_html = if cancelled {
                            AUTH_CANCELLED_HTML
                        } else {
                            AUTH_FAILED_HTML
                        };
                        if let Err(err) = write_html_response(&mut callback.stream, response_html).await {
                            warn!("Failed to write auth callback response: {}", err);
                        }
                        emit_auth_flow_event(
                            &provider_name,
                            if cancelled {
                                AuthFlowStatus::Cancelled
                            } else {
                                AuthFlowStatus::Failed
                            },
                            Some(message),
                        );
                        clear_auth_flow_state(flow_id);
                    }
                }
            }
            Err(AuthError::Cancelled) => {
                info!("Auth flow cancelled");
                if is_auth_flow_active(flow_id) {
                    emit_auth_flow_event(
                        &provider_name,
                        AuthFlowStatus::Cancelled,
                        Some("Sign-in was cancelled.".to_string()),
                    );
                    clear_auth_flow_state(flow_id);
                }
            }
            Err(err) => {
                error!("Auth callback listener failed: {}", err);
                if is_auth_flow_active(flow_id) {
                    emit_auth_flow_event(
                        &provider_name,
                        AuthFlowStatus::Failed,
                        Some(auth_flow_error_message(&err)),
                    );
                    clear_auth_flow_state(flow_id);
                }
            }
        }
    });

    Ok(())
}

fn build_authorize_url(provider: &str, state: &str) -> Result<String, AuthError> {
    let base_url = lock_r!(FDOLL)
        .app_config
        .api_base_url
        .clone()
        .ok_or(AuthError::InvalidConfig)?;

    let mut parsed = url::Url::parse(&base_url)
        .map_err(|e| AuthError::RequestFailed(format!("Invalid API base URL: {}", e)))?;
    let existing_path = parsed.path().trim_end_matches('/');
    parsed.set_path(&format!("{}/auth/sso/{}", existing_path, provider));
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("state", state)
        .finish();
    parsed.set_query(Some(&query));

    Ok(parsed.to_string())
}

async fn listen_for_callback(
    listener: TcpListener,
    cancel_token: CancellationToken,
) -> Result<PendingOAuthCallback, AuthError> {
    let timeout = tokio::time::Duration::from_secs(300);
    let start = tokio::time::Instant::now();

    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(AuthError::CallbackTimeout);
        }

        let remaining = timeout - elapsed;
        let accepted = tokio::select! {
            _ = cancel_token.cancelled() => return Err(AuthError::Cancelled),
            result = tokio::time::timeout(remaining, listener.accept()) => result,
        };

        let (mut stream, _) = match accepted {
            Ok(Ok(value)) => value,
            Ok(Err(err)) => {
                warn!("Accept error in auth callback listener: {}", err);
                continue;
            }
            Err(_) => return Err(AuthError::CallbackTimeout),
        };

        if let Some(params) = parse_callback(&mut stream).await? {
            return Ok(PendingOAuthCallback { stream, params });
        }
    }
}

async fn parse_callback(stream: &mut TcpStream) -> Result<Option<OAuthCallbackParams>, AuthError> {
    let mut buffer = [0; 4096];
    let bytes_read = match stream.read(&mut buffer).await {
        Ok(value) if value > 0 => value,
        _ => return Ok(None),
    };

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let first_line = request.lines().next().unwrap_or_default();
    let mut parts = first_line.split_whitespace();

    match (parts.next(), parts.next()) {
        (Some("GET"), Some(path)) if path.starts_with("/callback") => {
            let parsed = url::Url::parse(&format!("http://localhost{}", path))
                .map_err(|e| AuthError::RequestFailed(e.to_string()))?;
            let params: std::collections::HashMap<_, _> = parsed.query_pairs().into_owned().collect();

            let state = params
                .get("state")
                .cloned()
                .ok_or_else(|| AuthError::MissingParameter("state".to_string()))?;
            if let Some(error_code) = params.get("error") {
                let message = oauth_error_message(error_code, params.get("error_description"));
                return Ok(Some(OAuthCallbackParams {
                    state,
                    result: OAuthCallbackResult::Error {
                        message,
                        cancelled: is_oauth_cancellation(error_code),
                    },
                }));
            }

            let code = params
                .get("code")
                .cloned()
                .ok_or_else(|| AuthError::MissingParameter("code".to_string()))?;

            Ok(Some(OAuthCallbackParams {
                state,
                result: OAuthCallbackResult::Code(code),
            }))
        }
        (Some("GET"), Some("/health")) => {
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nOK")
                .await?;
            Ok(None)
        }
        _ => {
            stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n")
                .await?;
            Ok(None)
        }
    }
}

fn emit_auth_flow_event(provider: &str, status: AuthFlowStatus, message: Option<String>) {
    if let Err(err) = AuthFlowUpdated(AuthFlowUpdatedPayload {
        provider: provider.to_string(),
        status,
        message,
    })
    .emit(get_app_handle())
    {
        warn!("Failed to emit auth flow event: {}", err);
    }
}

fn auth_flow_error_message(err: &AuthError) -> String {
    match err {
        AuthError::Cancelled => "Sign-in was cancelled.".to_string(),
        AuthError::CallbackTimeout => "Sign-in timed out. Please try again.".to_string(),
        AuthError::MissingParameter(_) => {
            "Friendolls did not receive a complete sign-in response. Please try again.".to_string()
        }
        _ => "Friendolls could not complete sign-in. Please try again.".to_string(),
    }
}

fn oauth_error_message(error_code: &str, description: Option<&String>) -> String {
    if let Some(description) = description.filter(|description| !description.is_empty()) {
        return description.clone();
    }

    if is_oauth_cancellation(error_code) {
        "Sign-in was cancelled.".to_string()
    } else {
        "The sign-in provider reported an error. Please try again.".to_string()
    }
}

fn is_oauth_cancellation(error_code: &str) -> bool {
    matches!(error_code, "access_denied" | "user_cancelled" | "authorization_cancelled")
}

async fn write_html_response(stream: &mut TcpStream, html: &str) -> Result<(), AuthError> {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}
