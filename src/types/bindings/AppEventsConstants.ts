// Auto-generated constants - DO NOT EDIT
// Generated from Rust AppEvents enum

export const AppEvents = {
  CursorPosition: "cursor-position",
  SceneInteractive: "scene-interactive",
  AppDataRefreshed: "app-data-refreshed",
  SetInteractionOverlay: "set-interaction-overlay",
  EditDoll: "edit-doll",
  CreateDoll: "create-doll",
  UserStatusChanged: "user-status-changed",
  FriendCursorPosition: "friend-cursor-position",
  FriendDisconnected: "friend-disconnected",
  FriendActiveDollChanged: "friend-active-doll-changed",
  FriendUserStatus: "friend-user-status",
  InteractionReceived: "interaction-received",
  InteractionDeliveryFailed: "interaction-delivery-failed",
  FriendRequestReceived: "friend-request-received",
  FriendRequestAccepted: "friend-request-accepted",
  FriendRequestDenied: "friend-request-denied",
  Unfriended: "unfriended",
} as const;

export type AppEvents = typeof AppEvents[keyof typeof AppEvents];