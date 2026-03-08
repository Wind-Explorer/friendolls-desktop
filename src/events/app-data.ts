import { writable } from "svelte/store";
import { commands, events, type UserData } from "$lib/bindings";
import { createListenersSubscription, setupHmrCleanup } from "./listener-utils";

export const appData = writable<UserData | null>(null);

const subscription = createListenersSubscription();

/**
 * Starts listening for app data refresh events.
 * Initializes app data from the backend.
 */
export async function startAppData() {
  try {
    if (subscription.isListening()) return;
    appData.set(await commands.getAppData());
    const unlisten = await events.appDataRefreshed.listen((event) => {
      appData.set(event.payload);
    });
    subscription.addUnlisten(unlisten);
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
