use reqwest::StatusCode;
use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

use crate::{models::health::HealthError, remotes::health::HealthRemote};

/// Pings the server's health endpoint a maximum of
/// three times with a backoff of 500ms between
/// attempts. Return health error if no success.
pub async fn validate_server_health() -> Result<(), HealthError> {
    let health_remote = HealthRemote::try_new()?;

    // simple retry loop to smooth transient network issues
    const MAX_ATTEMPTS: u8 = 3;
    const BACKOFF_MS: u64 = 500;

    for attempt in 1..=MAX_ATTEMPTS {
        match health_remote.get_health().await {
            Ok(_) => {
                return Ok(());
            }
            Err(HealthError::NonOkStatus(status)) => {
                warn!(attempt, "server health reported non-OK status: {status}");
                return Err(HealthError::NonOkStatus(status));
            }
            Err(HealthError::UnexpectedStatus(status)) => {
                warn!(attempt, "server health check failed with status: {status}");
                return Err(HealthError::UnexpectedStatus(status));
            }
            Err(err) => {
                warn!(attempt, "server health check failed: {err}");
                if attempt == MAX_ATTEMPTS {
                    return Err(err);
                }
            }
        }

        if attempt < MAX_ATTEMPTS {
            sleep(Duration::from_millis(BACKOFF_MS)).await;
        }
    }

    warn!("Server is unavailable!");

    Err(HealthError::UnexpectedStatus(
        StatusCode::SERVICE_UNAVAILABLE,
    ))
}
