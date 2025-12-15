import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import type { CursorPosition } from "../types/bindings/CursorPosition";

export let cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

export type FriendCursorPosition = {
  userId: string;
  position: CursorPosition;
};

export let friendsCursorPositions = writable<Record<string, CursorPosition>>(
  {},
);

let unlistenCursor: UnlistenFn | null = null;
let unlistenFriendCursor: UnlistenFn | null = null;
let isListening = false;

/**
 * Initialize cursor tracking for this window.
 * Can be called from multiple windows - only the first call starts tracking on the Rust side,
 * but all windows can independently listen to the broadcast events.
 */
export async function initCursorTracking() {
  if (isListening) {
    return;
  }

  try {
    // Start tracking
    await invoke("start_cursor_tracking");

    // Listen to cursor position events (each window subscribes independently)
    unlistenCursor = await listen<CursorPositions>(
      "cursor-position",
      (event) => {
        cursorPositionOnScreen.set(event.payload);
      },
    );

    // Listen to friend cursor position events
    unlistenFriendCursor = await listen<[FriendCursorPosition]>(
      "friend-cursor-position",
      (event) => {
        // payload might be a string JSON if it comes directly from rust_socketio Payload::Text
        let payload = event.payload;

        if (typeof payload === "string") {
          try {
            payload = JSON.parse(payload);
          } catch (e) {
            console.error(
              "[Cursor] Failed to parse friend cursor position payload:",
              e,
            );
            return;
          }
        }

        // Rust socket.io client returns payload as an array of arguments
        // Since we only send one argument { userId, position }, it's the first element
        const data = Array.isArray(payload) ? payload[0] : payload;

        friendsCursorPositions.update((current) => {
          return {
            ...current,
            [data.userId]: data.position,
          };
        });
      },
    );

    isListening = true;
  } catch (err) {
    console.error("[Cursor] Failed to initialize cursor tracking:", err);
    throw err;
  }
}

/**
 * Stop listening to cursor events in this window.
 * Note: This doesn't stop the Rust-side tracking, just stops this window from receiving events.
 */
export function stopCursorTracking() {
  if (unlistenCursor) {
    unlistenCursor();
    unlistenCursor = null;
  }
  if (unlistenFriendCursor) {
    unlistenFriendCursor();
    unlistenFriendCursor = null;
  }
  isListening = false;
}

// Handle HMR (Hot Module Replacement) cleanup
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopCursorTracking();
  });
}
