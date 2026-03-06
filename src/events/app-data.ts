import { writable } from "svelte/store";
import { type UserData } from "../types/bindings/UserData";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import { createListenerSubscription, setupHmrCleanup } from "./listener-utils";

export const appData = writable<UserData | null>(null);

const subscription = createListenerSubscription();

/**
 * Starts listening for app data refresh events.
 * Initializes app data from the backend.
 */
export async function startAppData() {
  try {
    if (subscription.isListening()) return;
    appData.set(await invoke("get_app_data"));
    const unlisten = await listen<UserData>(
      AppEvents.AppDataRefreshed,
      (event) => {
        appData.set(event.payload);
      },
    );
    subscription.setUnlisten(unlisten);
    subscription.setListening(true);
  } catch (error) {
    console.error(error);
    throw error;
  }
}

export function stopAppData() {
  subscription.stop();
}

setupHmrCleanup(stopAppData);
