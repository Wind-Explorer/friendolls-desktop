import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable } from "svelte/store";
import type { AppMetadata } from "../types/bindings/AppMetadata";

export type FriendUserStatus = {
  appMetadata: AppMetadata;
  state: "idle" | "resting";
};

export const friendsUserStatuses = writable<Record<string, FriendUserStatus>>({});

let unlistenStatus: UnlistenFn | null = null;
let unlistenFriendDisconnected: UnlistenFn | null = null;
let isListening = false;

export async function initUserStatusListeners() {
  if (isListening) return;

  try {
    unlistenStatus = await listen<unknown>("friend-user-status", (event) => {
      let payload = event.payload as any;
      if (typeof payload === "string") {
        try {
          payload = JSON.parse(payload);
        } catch (error) {
          console.error("Failed to parse friend-user-status payload", error);
          return;
        }
      }

      const userId = payload?.userId as string | undefined;
      const status = payload?.status as FriendUserStatus | undefined;

      if (!userId || !status) return;
      if (!status.appMetadata) return;
      
      // Validate that appMetadata has at least one valid name
      const hasValidName = 
        (typeof status.appMetadata.localized === "string" && status.appMetadata.localized.trim() !== "") ||
        (typeof status.appMetadata.unlocalized === "string" && status.appMetadata.unlocalized.trim() !== "");
      if (!hasValidName) return;
      
      if (status.state !== "idle" && status.state !== "resting") return;

      friendsUserStatuses.update((current) => ({
        ...current,
        [userId]: {
          appMetadata: status.appMetadata,
          state: status.state,
        },
      }));
    });

    unlistenFriendDisconnected = await listen<[{ userId: string }] | { userId: string } | string>(
      "friend-disconnected",
      (event) => {
        let payload = event.payload as any;
        if (typeof payload === "string") {
          try {
            payload = JSON.parse(payload);
          } catch (error) {
            console.error("Failed to parse friend-disconnected payload", error);
            return;
          }
        }

        const data = Array.isArray(payload) ? payload[0] : payload;
        const userId = data?.userId as string | undefined;
        if (!userId) return;

        friendsUserStatuses.update((current) => {
          const next = { ...current };
          delete next[userId];
          return next;
        });
      },
    );

    isListening = true;
  } catch (error) {
    console.error("Failed to initialize user status listeners", error);
    throw error;
  }
}

export function stopUserStatusListeners() {
  if (unlistenStatus) {
    unlistenStatus();
    unlistenStatus = null;
  }
  if (unlistenFriendDisconnected) {
    unlistenFriendDisconnected();
    unlistenFriendDisconnected = null;
  }
  isListening = false;
}

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    stopUserStatusListeners();
  });
}
