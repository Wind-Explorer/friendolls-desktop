use tracing::warn;

use crate::{
    models::interaction::SendInteractionDto,
    services::ws::{ws_emit_soft, WS_EVENT},
};

pub async fn send_interaction(dto: SendInteractionDto) -> Result<(), String> {
    ws_emit_soft(WS_EVENT::CLIENT_SEND_INTERACTION, dto)
        .await
        .map_err(|err| {
            warn!("Failed to send interaction: {}", err);
            err
        })
}
