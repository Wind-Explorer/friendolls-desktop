use crate::{
    init::lifecycle::{handle_disasterous_failure, validate_server_health},
    lock_w,
    services::ws::client::establish_websocket_connection,
    state::FDOLL,
};
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// Starts a periodic health monitor that validates server connectivity
/// and attempts to recover WebSocket connection if health checks fail.
pub async fn start_health_monitor() {
    stop_health_monitor();
    
    let cancel_token = CancellationToken::new();
    {
        let mut guard = lock_w!(FDOLL);
        guard.network.health_monitor_token = Some(cancel_token.clone());
    }

    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(30)); // Check every 30 seconds
        let mut consecutive_failures = 0u8;
        const MAX_FAILURES: u8 = 3;

        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    info!("Health monitor stopped");
                    break;
                }
                _ = interval.tick() => {
                    match validate_server_health().await {
                        Ok(_) => {
                            consecutive_failures = 0;
                        }
                        Err(e) => {
                            consecutive_failures = consecutive_failures.saturating_add(1);
                            warn!(
                                "Health check failed ({}/{}): {}",
                                consecutive_failures, MAX_FAILURES, e
                            );

                            if consecutive_failures >= MAX_FAILURES {
                                info!("Server appears unreachable after {} attempts, triggering recovery", MAX_FAILURES);
                                handle_disasterous_failure(Some(format!(
                                    "Lost connection to server: {}",
                                    e
                                )))
                                .await;
                                break;
                            } else {
                                // Try to re-establish WebSocket connection
                                info!("Attempting to re-establish WebSocket connection");
                                establish_websocket_connection().await;
                            }
                        }
                    }
                }
            }
        }
    });
}

/// Stops the health monitor loop.
pub fn stop_health_monitor() {
    let mut guard = lock_w!(FDOLL);
    if let Some(token) = guard.network.health_monitor_token.take() {
        token.cancel();
    }
}
