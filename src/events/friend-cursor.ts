import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import type { DollDto } from "../types/bindings/DollDto";
import type { FriendDisconnectedPayload } from "../types/bindings/FriendDisconnectedPayload";
import type { FriendActiveDollChangedPayload } from "../types/bindings/FriendActiveDollChangedPayload";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

export type FriendCursorPosition = {
  userId: string;
  position: CursorPositions;
};

type FriendCursorData = {
  position: CursorPositions;
  lastUpdated: number;
};

export const friendsCursorPositions = writable<Record<string, CursorPositions>>(
  {},
);
export const friendsActiveDolls = writable<Record<string, DollDto | null>>({});

const subscription = createMultiListenerSubscription();

let friendCursorState: Record<string, FriendCursorData> = {};

/**
 * Starts listening for friend cursor position and active doll changes.
 * Also handles friend disconnection events.
 */
export async function startFriendCursorTracking() {
  if (subscription.isListening()) return;

  try {
    // TODO: Add initial sync for existing friends' cursors and dolls if needed

    const unlistenFriendCursor = await listen<FriendCursorPosition>(
      AppEvents.FriendCursorPosition,
      (event) => {
        const data = event.payload;

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
      },
    );
    subscription.addUnlisten(unlistenFriendCursor);

    const unlistenFriendDisconnected = await listen<FriendDisconnectedPayload>(
      AppEvents.FriendDisconnected,
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
      await listen<FriendActiveDollChangedPayload>(
        AppEvents.FriendActiveDollChanged,
        (event) => {
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
        },
      );
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
