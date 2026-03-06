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
} as const;

export type AppEvents = typeof AppEvents[keyof typeof AppEvents];