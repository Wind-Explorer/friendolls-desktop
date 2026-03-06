import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { CursorPositions } from "../types/bindings/CursorPositions";
import type { DollDto } from "../types/bindings/DollDto";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  parseEventPayload,
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

  const unlistenFriendDisconnected = await listen<
    [{ userId: string }] | { userId: string } | string
  >(AppEvents.FriendDisconnected, (event) => {
    const payload = parseEventPayload<
      [{ userId: string }] | { userId: string }
    >(event.payload, AppEvents.FriendDisconnected);
    if (!payload) return;

    const data = Array.isArray(payload) ? payload[0] : payload;

    if (friendCursorState[data.userId]) {
      delete friendCursorState[data.userId];
    }

    friendsCursorPositions.update((current) =>
      removeFromStore(current, data.userId),
    );
  });
  subscription.addUnlisten(unlistenFriendDisconnected);

  const unlistenFriendActiveDollChanged = await listen<
    | string
    | {
        friendId: string;
        doll: DollDto | null;
      }
  >(AppEvents.FriendActiveDollChanged, (event) => {
    const data = parseEventPayload<{
      friendId: string;
      doll: DollDto | null;
    }>(event.payload, AppEvents.FriendActiveDollChanged);
    if (!data) return;

    const payload = data as { friendId: string; doll: DollDto | null };

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
          [payload.friendId]: payload.doll!,
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
