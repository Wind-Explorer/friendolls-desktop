import { writable } from "svelte/store";
import { commands, events } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const sceneInteractive = writable<boolean>(false);

export const { start: startSceneInteractive, stop: stopSceneInteractive } =
  createEventSource(async (addEventListener) => {
    sceneInteractive.set(await commands.getSceneInteractive());
    addEventListener(
      await events.sceneInteractiveChanged.listen((event) => {
        sceneInteractive.set(Boolean(event.payload));
      }),
    );
  });
