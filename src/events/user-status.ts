import { writable } from "svelte/store";
import { events, type UserStatusPayload } from "$lib/bindings";
import {
  createListenersSubscription,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

export const friendsPresenceStates = writable<
  Record<string, UserStatusPayload>
>({});
export const currentPresenceState = writable<UserStatusPayload | null>(null);

const subscription = createListenersSubscription();

/**
 * Starts listening for user status changes and friend status updates.
 */
export async function startUserStatus() {
  if (subscription.isListening()) return;

  try {
    const unlistenStatus = await events.friendUserStatusChanged.listen(
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

    const unlistenUserStatusChanged = await events.userStatusChanged.listen(
      (event) => {
        currentPresenceState.set(event.payload);
      },
    );
    subscription.addUnlisten(unlistenUserStatusChanged);

    const unlistenFriendDisconnected = await events.friendDisconnected.listen(
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
