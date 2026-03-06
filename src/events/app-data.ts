import { writable } from "svelte/store";
import { type UserData } from "../types/bindings/UserData";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import { createListenerSubscription, setupHmrCleanup } from "./listener-utils";

export const appData = writable<UserData | null>(null);

const subscription = createListenerSubscription();

export async function initAppDataListener() {
  try {
    if (subscription.isListening()) return;
    appData.set(await invoke("get_app_data"));
    const unlisten = await listen<UserData>(
      AppEvents.AppDataRefreshed,
      (event) => {
        console.log("app-data-refreshed", event.payload);
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

export function stopAppDataListener() {
  subscription.stop();
}

setupHmrCleanup(stopAppDataListener);
