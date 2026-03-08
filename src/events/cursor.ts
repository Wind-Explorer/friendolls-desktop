import { writable } from "svelte/store";
import { events, type CursorPositions } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

export const { start: startCursorTracking, stop: stopCursorTracking } =
  createEventSource(async (addEventListener) => {
    addEventListener(
      await events.cursorMoved.listen((event) => {
        cursorPositionOnScreen.set(event.payload);
      }),
    );
  });
