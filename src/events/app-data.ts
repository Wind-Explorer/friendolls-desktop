import { writable } from "svelte/store";
import { type UserData } from "../types/bindings/UserData";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export let appData = writable<UserData | null>(null);

let unlisten: UnlistenFn | null = null;
let isListening = false;

export async function initAppDataListener() {
  try {
    if (isListening) return;
    appData.set(await invoke("get_app_data"));
    unlisten = await listen<UserData>("app-data-refreshed", (event) => {
      console.log("app-data-refreshed", event.payload);
      appData.set(event.payload);
    });

    isListening = true;
  } catch (error) {
    console.error(error);
    throw error;
  }
}

export function stopAppDataListener() {
  if (unlisten) {
    unlisten();
    unlisten = null;
    isListening = false;
  }
}

// Handle HMR (Hot Module Replacement) cleanup
if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopAppDataListener();
  });
}
