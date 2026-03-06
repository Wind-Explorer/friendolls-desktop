import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import { AppEvents } from "../types/bindings/AppEventsConstants";

export const sceneInteractive = writable<boolean>(false);

let unlisten: UnlistenFn | null = null;
let isListening = false;

export async function initSceneInteractiveListener() {
  if (isListening) return;

  try {
    // ensure initial default matches backend default
    sceneInteractive.set(false);
    unlisten = await listen<boolean>(AppEvents.SceneInteractive, (event) => {
      sceneInteractive.set(Boolean(event.payload));
    });
    isListening = true;
  } catch (error) {
    console.error("Failed to initialize scene interactive listener:", error);
    throw error;
  }
}

export function stopSceneInteractiveListener() {
  if (unlisten) {
    unlisten();
    unlisten = null;
    isListening = false;
  }
}

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopSceneInteractiveListener();
  });
}
