import { writable } from "svelte/store";
import {
  commands,
  events,
  type AppState,
  type NekoPosition,
} from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export type NeksPosition = NekoPosition;
export type { AppState };

const initialState: AppState = {
  sceneSetup: {
    nekosPosition: null,
    nekosOpacity: 1,
    nekosScale: 1,
  },
};

export const appState = writable<AppState>(initialState);

export const { start: startAppState, stop: stopAppState } = createEventSource(
  async (addEventListener) => {
    appState.set(await commands.getAppState());
    addEventListener(
      await events.appStateChanged.listen((event) => {
        appState.set(event.payload);
      }),
    );
  },
);
