import { writable } from "svelte/store";
import { commands, events } from "$lib/bindings";
import { createListenersSubscription, setupHmrCleanup } from "./listener-utils";

export const sceneInteractive = writable<boolean>(false);

const subscription = createListenersSubscription();

/**
 * Starts listening for scene interactive state changes.
 * Initializes the scene interactive state from the backend.
 */
export async function startSceneInteractive() {
  if (subscription.isListening()) return;

  try {
    sceneInteractive.set(await commands.getSceneInteractive());
    const unlisten = await events.sceneInteractiveChanged.listen((event) => {
      sceneInteractive.set(Boolean(event.payload));
    });
    subscription.addUnlisten(unlisten);
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
