import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { FriendDisconnectedPayload } from "../types/bindings/FriendDisconnectedPayload";
import type { FriendUserStatusPayload } from "../types/bindings/FriendUserStatusPayload";
import type { UserStatusPayload } from "../types/bindings/UserStatusPayload";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

export const friendsPresenceStates = writable<
  Record<string, UserStatusPayload>
>({});
export const currentPresenceState = writable<UserStatusPayload | null>(null);

const subscription = createMultiListenerSubscription();

/**
 * Starts listening for user status changes and friend status updates.
 */
export async function startUserStatus() {
  if (subscription.isListening()) return;

  try {
    const unlistenStatus = await listen<FriendUserStatusPayload>(
      AppEvents.FriendUserStatus,
      (event) => {
        const { userId, status } = event.payload;

        const hasValidName =
          (typeof status.presenceStatus.title === "string" &&
            status.presenceStatus.title.trim() !== "") ||
          (typeof status.presenceStatus.subtitle === "string" &&
            status.presenceStatus.subtitle.trim() !== "");
        if (!hasValidName) return;

        friendsPresenceStates.update((current) => ({
          ...current,
          [userId]: status,
        }));
      },
    );
    subscription.addUnlisten(unlistenStatus);

    const unlistenUserStatusChanged = await listen<UserStatusPayload>(
      AppEvents.UserStatusChanged,
      (event) => {
        currentPresenceState.set(event.payload);
      },
    );
    subscription.addUnlisten(unlistenUserStatusChanged);

    const unlistenFriendDisconnected = await listen<FriendDisconnectedPayload>(
      AppEvents.FriendDisconnected,
      (event) => {
        const { userId } = event.payload;
        friendsPresenceStates.update((current) =>
          removeFromStore(current, userId),
        );
      },
    );
    subscription.addUnlisten(unlistenFriendDisconnected);

    subscription.setListening(true);
  } catch (error) {
    console.error("Failed to initialize user status listeners", error);
    throw error;
  }
}

export function stopUserStatus() {
  subscription.stop();
}

setupHmrCleanup(stopUserStatus);
