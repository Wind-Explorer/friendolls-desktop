import { writable } from "svelte/store";
import { commands, events, type NekoPositionsDto } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const nekoPositions = writable<NekoPositionsDto>({});

export const { start: startNekoPositions, stop: stopNekoPositions } =
  createEventSource(async (addEventListener) => {
    nekoPositions.set(await commands.getNekoPositions());

    addEventListener(
      await events.nekoPositionsUpdated.listen((event) => {
        nekoPositions.set(event.payload);
      }),
    );
  });
