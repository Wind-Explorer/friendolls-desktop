import { writable } from "svelte/store";
import { commands, events, type UserData } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const appData = writable<UserData | null>(null);

export const { start: startAppData, stop: stopAppData } = createEventSource(
  async (addEventListener) => {
    appData.set(await commands.getAppData());
    addEventListener(
      await events.appDataRefreshed.listen((event) => {
        appData.set(event.payload);
      }),
    );
  },
);
