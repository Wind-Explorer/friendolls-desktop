import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import { createListenerSubscription, setupHmrCleanup } from "./listener-utils";

export const sceneInteractive = writable<boolean>(false);

const subscription = createListenerSubscription();

/**
 * Starts listening for scene interactive state changes.
 * Initializes the scene interactive state from the backend.
 */
export async function startSceneInteractive() {
  if (subscription.isListening()) return;

  try {
    sceneInteractive.set(await invoke("get_scene_interactive"));
    const unlisten = await listen<boolean>(
      AppEvents.SceneInteractive,
      (event) => {
        sceneInteractive.set(Boolean(event.payload));
      },
    );
    subscription.setUnlisten(unlisten);
    subscription.setListening(true);
  } catch (error) {
    console.error("Failed to initialize scene interactive listener:", error);
    throw error;
  }
}

export function stopSceneInteractive() {
  subscription.stop();
}

setupHmrCleanup(stopSceneInteractive);
