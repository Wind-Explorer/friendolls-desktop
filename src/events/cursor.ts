import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import type { CursorPosition } from "../types/bindings/CursorPosition";
import type { DollDto } from "../types/bindings/DollDto";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  parseEventPayload,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

export const cursorPositionOnScreen = writable<CursorPositions>({
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
export const friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);
export const friendsActiveDolls = writable<Record<string, DollDto | null>>({});

const subscription = createMultiListenerSubscription();

// Internal state to track timestamps
let friendCursorState: Record<string, FriendCursorData> = {};

/**
 * Initialize cursor tracking for this window.
 * Can be called from multiple windows - only the first call starts tracking on the Rust side,
 * but all windows can independently listen to the broadcast events.
 */
export async function initCursorTracking() {
  if (subscription.isListening()) return;

  try {
    // Listen to cursor position events (each window subscribes independently)
    const unlistenCursor = await listen<CursorPositions>(
      AppEvents.CursorPosition,
      (event) => {
        cursorPositionOnScreen.set(event.payload);
      },
    );
    subscription.addUnlisten(unlistenCursor);

    // Listen to friend cursor position events
    const unlistenFriendCursor = await listen<FriendCursorPosition>(
      AppEvents.FriendCursorPosition,
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
    subscription.addUnlisten(unlistenFriendCursor);

    // Listen to friend disconnected events
    const unlistenFriendDisconnected = await listen<
      [{ userId: string }] | { userId: string } | string
    >(AppEvents.FriendDisconnected, (event) => {
      const payload = parseEventPayload<
        [{ userId: string }] | { userId: string }
      >(event.payload, "friend-disconnected");
      if (!payload) return;

      const data = Array.isArray(payload) ? payload[0] : payload;

      // Remove from internal state
      if (friendCursorState[data.userId]) {
        delete friendCursorState[data.userId];
      }

      // Update svelte store
      friendsCursorPositions.update((current) =>
        removeFromStore(current, data.userId),
      );
    });
    subscription.addUnlisten(unlistenFriendDisconnected);

    // Listen to friend active doll changed events
    const unlistenFriendActiveDollChanged = await listen<
      | string
      | {
          friendId: string;
          doll: DollDto | null;
        }
    >(AppEvents.FriendActiveDollChanged, (event) => {
      const data = parseEventPayload<{
        friendId: string;
        doll: DollDto | null;
      }>(event.payload, "friend-active-doll-changed");
      if (!data) return;

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
        friendsCursorPositions.update((current) =>
          removeFromStore(current, payload.friendId),
        );
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
    subscription.addUnlisten(unlistenFriendActiveDollChanged);

    subscription.setListening(true);
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
  subscription.stop();
}

setupHmrCleanup(stopCursorTracking);
