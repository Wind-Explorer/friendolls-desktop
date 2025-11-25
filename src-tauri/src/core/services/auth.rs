use crate::{core::state::FDOLL, lock_r, APP_HANDLE};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distr::Alphanumeric, Rng};
use sha2::{Digest, Sha256};
use tauri_plugin_opener::OpenerExt;

/// Generate a random code verifier (PKCE spec: 43 to 128 chars, here defaulting to 64)
fn generate_code_verifier(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generate code challenge from a code verifier
fn generate_code_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let result = hasher.finalize();
    URL_SAFE_NO_PAD.encode(&result)
}

/// Returns the auth pass object, including
/// access token, refresh token, expire time etc.
#[allow(dead_code)]
pub fn get_tokens() {
    todo!();
}

/// Opens the auth portal in the browser,
/// and returns auth code after user logged in.
pub fn get_auth_code() {
    let app_config = lock_r!(FDOLL)
        .app_config
        .clone()
        .expect("Invalid app config");

    let opener = APP_HANDLE.get().unwrap().opener();

    let code_verifier = generate_code_verifier(64);
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_code_verifier(16);

    let mut url = url::Url::parse(&app_config.auth.auth_url.as_str()).expect("Invalid app config");
    url.query_pairs_mut()
        .append_pair("client_id", &app_config.auth.audience.as_str())
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &app_config.auth.redirect_uri.as_str())
        .append_pair("scope", "openid email profile")
        .append_pair("state", &state)
        .append_pair("code_challenge", &code_challenge)
        .append_pair("code_challenge_method", "S256");

    match opener.open_url(url, None::<&str>) {
        Ok(_) => (),
        Err(e) => panic!("Failed to open auth portal: {}", e),
    }
}

/// Accepts a refresh token and
/// returns a new access token.
#[allow(dead_code)]
pub fn refresh_token() {
    todo!();
}
