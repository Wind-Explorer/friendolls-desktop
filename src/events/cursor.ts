import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import type { CursorPosition } from "../types/bindings/CursorPosition";
import type { DollDto } from "../types/bindings/DollDto";
import { AppEvents } from "../types/bindings/AppEventsConstants";

export let cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

export type FriendCursorPosition = {
  userId: string;
  position: CursorPositions;
};

// Map of userId -> { position: CursorPositions, lastUpdated: number }
// We store the timestamp to detect stale cursors
type FriendCursorData = {
  position: CursorPositions;
  lastUpdated: number;
};

// The exported store will only expose the position part to consumers,
// but internally we manage the full data.
// Actually, it's easier if we just export the positions and manage state internally.
export let friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);
export let friendsActiveDolls = writable<Record<string, DollDto | null>>({});

let unlistenCursor: UnlistenFn | null = null;
let unlistenFriendCursor: UnlistenFn | null = null;
let unlistenFriendDisconnected: UnlistenFn | null = null;
let unlistenFriendActiveDollChanged: UnlistenFn | null = null;
let isListening = false;

// Internal state to track timestamps
let friendCursorState: Record<string, FriendCursorData> = {};

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
    // Listen to cursor position events (each window subscribes independently)
    unlistenCursor = await listen<CursorPositions>(
      AppEvents.CursorPosition,
      (event) => {
        cursorPositionOnScreen.set(event.payload);
      },
    );

    // Listen to friend cursor position events
    unlistenFriendCursor = await listen<FriendCursorPosition>(
      "friend-cursor-position",
      (event) => {
        // We now receive a clean object from Rust
        const data = event.payload;

        // Update internal state with timestamp
        friendCursorState[data.userId] = {
          position: data.position,
          lastUpdated: Date.now(),
        };

        friendsCursorPositions.update((current) => {
          return {
            ...current,
            [data.userId]: data.position,
          };
        });
      },
    );

    // Listen to friend disconnected events
    unlistenFriendDisconnected = await listen<[{ userId: string }]>(
      "friend-disconnected",
      (event) => {
        let payload = event.payload;
        if (typeof payload === "string") {
          try {
            payload = JSON.parse(payload);
          } catch (e) {
            console.error("Failed to parse friend disconnected payload:", e);
            return;
          }
        }

        const data = Array.isArray(payload) ? payload[0] : payload;

        // Remove from internal state
        if (friendCursorState[data.userId]) {
          delete friendCursorState[data.userId];
        }

        // Update svelte store
        friendsCursorPositions.update((current) => {
          const next = { ...current };
          delete next[data.userId];
          return next;
        });
      },
    );

    // Listen to friend active doll changed events
    unlistenFriendActiveDollChanged = await listen<
      | string
      | {
          friendId: string;
          doll: DollDto | null;
        }
    >("friend-active-doll-changed", (event) => {
      let data = event.payload;

      if (typeof data === "string") {
        try {
          data = JSON.parse(data);
        } catch (e) {
          console.error(
            "Failed to parse friend-active-doll-changed payload:",
            e,
          );
          return;
        }
      }

      // Cast to expected type after parsing
      const payload = data as { friendId: string; doll: DollDto | null };

      if (!payload.doll) {
        // If doll is null, it means the friend deactivated their doll.

        // Update the active dolls store to explicitly set this friend's doll to null
        // We MUST set it to null instead of deleting it, otherwise the UI might
        // fall back to the initial appData snapshot which might still have the old doll.
        friendsActiveDolls.update((current) => {
          const next = { ...current };
          next[payload.friendId] = null;
          return next;
        });

        // Also remove from cursor positions so the sprite disappears
        friendsCursorPositions.update((current) => {
          const next = { ...current };
          delete next[payload.friendId];
          return next;
        });
      } else {
        // Update or add the new doll configuration
        friendsActiveDolls.update((current) => {
          return {
            ...current,
            [payload.friendId]: payload.doll!,
          };
        });
      }
    });

    isListening = true;
  } catch (err) {
    console.error("Failed to initialize cursor tracking:", err);
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
  if (unlistenFriendDisconnected) {
    unlistenFriendDisconnected();
    unlistenFriendDisconnected = null;
  }
  if (unlistenFriendActiveDollChanged) {
    unlistenFriendActiveDollChanged();
    unlistenFriendActiveDollChanged = null;
  }
  isListening = false;
}

// Handle HMR (Hot Module Replacement) cleanup
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopCursorTracking();
  });
}
