import { writable } from "svelte/store";
import { events, type CursorPositions } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);

// Here for now. Will extract into shared
// util when there's more similar cases.
function toCursorPositionsRecord(
  payload: Partial<Record<string, CursorPositions>>,
): Record<string, CursorPositions> {
  return Object.fromEntries(
    Object.entries(payload).filter(
      (entry): entry is [string, CursorPositions] => {
        return entry[1] !== undefined;
      },
    ),
  );
}

export const {
  start: startFriendCursorTracking,
  stop: stopFriendCursorTracking,
} = createEventSource(async (addEventListener) => {
  addEventListener(
    await events.friendCursorPositionsUpdated.listen((event) => {
      friendsCursorPositions.set(toCursorPositionsRecord(event.payload));
    }),
  );
});
