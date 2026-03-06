import { listen } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { PresenceStatus } from "../types/bindings/PresenceStatus";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  parseEventPayload,
  removeFromStore,
  setupHmrCleanup,
} from "./listener-utils";

export type UserStatus = {
  presenceStatus: PresenceStatus;
  state: "idle" | "resting";
};

export const friendsUserStatuses = writable<Record<string, UserStatus>>({});

const subscription = createMultiListenerSubscription();

export async function initUserStatusListeners() {
  if (subscription.isListening()) return;

  try {
    const unlistenStatus = await listen<unknown>(
      AppEvents.FriendUserStatus,
      (event) => {
        const payload = parseEventPayload<{
          userId?: string;
          status?: UserStatus;
        }>(event.payload, "friend-user-status");
        if (!payload) return;

        const userId = payload.userId;
        const status = payload.status;

        if (!userId || !status) return;
        if (!status.presenceStatus) return;

        // Validate that appMetadata has at least one valid name
        const hasValidName =
          (typeof status.presenceStatus.title === "string" &&
            status.presenceStatus.title.trim() !== "") ||
          (typeof status.presenceStatus.subtitle === "string" &&
            status.presenceStatus.subtitle.trim() !== "");
        if (!hasValidName) return;

        if (status.state !== "idle" && status.state !== "resting") return;

        friendsUserStatuses.update((current) => ({
          ...current,
          [userId]: {
            presenceStatus: status.presenceStatus,
            state: status.state,
          },
        }));
      },
    );
    subscription.addUnlisten(unlistenStatus);

    const unlistenFriendDisconnected = await listen<
      [{ userId: string }] | { userId: string } | string
    >(AppEvents.FriendDisconnected, (event) => {
      const payload = parseEventPayload<
        [{ userId: string }] | { userId: string }
      >(event.payload, "friend-disconnected");
      if (!payload) return;

      const data = Array.isArray(payload) ? payload[0] : payload;
      const userId = data?.userId as string | undefined;
      if (!userId) return;

      friendsUserStatuses.update((current) => removeFromStore(current, userId));
    });
    subscription.addUnlisten(unlistenFriendDisconnected);

    subscription.setListening(true);
  } catch (error) {
    console.error("Failed to initialize user status listeners", error);
    throw error;
  }
}

export function stopUserStatusListeners() {
  subscription.stop();
}

setupHmrCleanup(stopUserStatusListeners);
