import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import { createListenerSubscription, setupHmrCleanup } from "./listener-utils";

export const cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

const subscription = createListenerSubscription();

/**
 * Starts tracking the local cursor position.
 * Initializes cursor position from the backend and listens for updates.
 */
export async function startCursorTracking() {
  if (subscription.isListening()) return;

  try {
    const unlisten = await listen<CursorPositions>(
      AppEvents.CursorPosition,
      (event) => {
        cursorPositionOnScreen.set(event.payload);
      },
    );
    subscription.setUnlisten(unlisten);
    subscription.setListening(true);
  } catch (err) {
    console.error("Failed to initialize cursor tracking:", err);
    throw err;
  }
}

export function stopCursorTracking() {
  subscription.stop();
}

setupHmrCleanup(stopCursorTracking);
