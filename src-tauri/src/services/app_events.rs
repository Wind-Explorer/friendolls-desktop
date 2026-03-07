use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use crate::{
    models::{
        app_data::UserData,
        event_payloads::{
            FriendActiveDollChangedPayload, FriendDisconnectedPayload,
            FriendRequestAcceptedPayload, FriendRequestDeniedPayload, FriendRequestReceivedPayload,
            FriendUserStatusPayload, UnfriendedPayload, UserStatusPayload,
        },
        interaction::{InteractionDeliveryFailedDto, InteractionPayloadDto},
    },
    services::{cursor::CursorPositions, ws::OutgoingFriendCursorPayload},
};

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "cursor-position")]
pub struct CursorMoved(pub CursorPositions);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "scene-interactive")]
pub struct SceneInteractiveChanged(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "app-data-refreshed")]
pub struct AppDataRefreshed(pub UserData);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "set-interaction-overlay")]
pub struct SetInteractionOverlay(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "edit-doll")]
pub struct EditDoll(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "create-doll")]
pub struct CreateDoll;

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "user-status-changed")]
pub struct UserStatusChanged(pub UserStatusPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-cursor-position")]
pub struct FriendCursorPositionUpdated(pub OutgoingFriendCursorPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-disconnected")]
pub struct FriendDisconnected(pub FriendDisconnectedPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-active-doll-changed")]
pub struct FriendActiveDollChanged(pub FriendActiveDollChangedPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-user-status")]
pub struct FriendUserStatusChanged(pub FriendUserStatusPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "interaction-received")]
pub struct InteractionReceived(pub InteractionPayloadDto);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "interaction-delivery-failed")]
pub struct InteractionDeliveryFailed(pub InteractionDeliveryFailedDto);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-request-received")]
pub struct FriendRequestReceived(pub FriendRequestReceivedPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-request-accepted")]
pub struct FriendRequestAccepted(pub FriendRequestAcceptedPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "friend-request-denied")]
pub struct FriendRequestDenied(pub FriendRequestDeniedPayload);

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[tauri_specta(event_name = "unfriended")]
pub struct Unfriended(pub UnfriendedPayload);
