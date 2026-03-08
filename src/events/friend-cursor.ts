import { writable } from "svelte/store";
import {
  events,
  type CursorPositions,
  type DollDto,
  type OutgoingFriendCursorPayload,
} from "$lib/bindings";
import { createEventSource, removeFromStore } from "./listener-utils";

type FriendCursorData = {
  position: CursorPositions;
  lastUpdated: number;
};

export const friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);
export const friendsActiveDolls = writable<Record<string, DollDto | null>>({});

export const {
  start: startFriendCursorTracking,
  stop: stopFriendCursorTracking,
} = createEventSource(async (addEventListener) => {
  let friendCursorState: Record<string, FriendCursorData> = {};
  addEventListener(
    await events.friendCursorPositionUpdated.listen((event) => {
      const data: OutgoingFriendCursorPayload = event.payload;

      friendCursorState[data.userId] = {
        position: data.position,
        lastUpdated: Date.now(),
      };

      friendsCursorPositions.update((current) => {
        return {
          ...current,
          [data.userId]: data.position,
        };
      });
    }),
  );

  addEventListener(
    await events.friendDisconnected.listen((event) => {
      const data = event.payload;

      if (friendCursorState[data.userId]) {
        delete friendCursorState[data.userId];
      }

      friendsCursorPositions.update((current) =>
        removeFromStore(current, data.userId),
      );
    }),
  );

  addEventListener(
    await events.friendActiveDollChanged.listen((event) => {
      const payload = event.payload;

      if (!payload.doll) {
        friendsActiveDolls.update((current) => {
          const next = { ...current };
          next[payload.friendId] = null;
          return next;
        });

        friendsCursorPositions.update((current) =>
          removeFromStore(current, payload.friendId),
        );
      } else {
        friendsActiveDolls.update((current) => {
          return {
            ...current,
            [payload.friendId]: payload.doll,
          };
        });
      }
    }),
  );
});
