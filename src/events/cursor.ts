import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";

export let cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

let unlisten: UnlistenFn | null = null;
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
    unlisten = await listen<CursorPositions>("cursor-position", (event) => {
      cursorPositionOnScreen.set(event.payload);
    });

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
  if (unlisten) {
    unlisten();
    unlisten = null;
    isListening = false;
  }
}

// Handle HMR (Hot Module Replacement) cleanup
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopCursorTracking();
  });
}
