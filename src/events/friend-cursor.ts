import { writable } from "svelte/store";
import {
  events,
  type CursorPositions,
  type DollDto,
  type OutgoingFriendCursorPayload,
} from "$lib/bindings";
import {
  createListenersSubscription,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

type FriendCursorData = {
  position: CursorPositions;
  lastUpdated: number;
};

export const friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);
export const friendsActiveDolls = writable<Record<string, DollDto | null>>({});

const subscription = createListenersSubscription();

let friendCursorState: Record<string, FriendCursorData> = {};

/**
 * Starts listening for friend cursor position and active doll changes.
 * Also handles friend disconnection events.
 */
export async function startFriendCursorTracking() {
  if (subscription.isListening()) return;

  try {
    // TODO: Add initial sync for existing friends' cursors and dolls if needed

    const unlistenFriendCursor =
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
      });
    subscription.addUnlisten(unlistenFriendCursor);

    const unlistenFriendDisconnected = await events.friendDisconnected.listen(
      (event) => {
        const data = event.payload;

        if (friendCursorState[data.userId]) {
          delete friendCursorState[data.userId];
        }

        friendsCursorPositions.update((current) =>
          removeFromStore(current, data.userId),
        );
      },
    );
    subscription.addUnlisten(unlistenFriendDisconnected);

    const unlistenFriendActiveDollChanged =
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
      });
    subscription.addUnlisten(unlistenFriendActiveDollChanged);

    subscription.setListening(true);
  } catch (err) {
    console.error("Failed to initialize friend cursor tracking:", err);
    throw err;
  }
}

export function stopFriendCursorTracking() {
  subscription.stop();
}

setupHmrCleanup(stopFriendCursorTracking);
