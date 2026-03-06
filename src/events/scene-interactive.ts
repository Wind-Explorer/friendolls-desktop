import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import { createListenerSubscription, setupHmrCleanup } from "./listener-utils";

export const sceneInteractive = writable<boolean>(false);

const subscription = createListenerSubscription();

export async function initSceneInteractiveListener() {
  if (subscription.isListening()) return;

  try {
    // ensure initial default matches backend default
    sceneInteractive.set(false);
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

export function stopSceneInteractiveListener() {
  subscription.stop();
}

setupHmrCleanup(stopSceneInteractiveListener);
