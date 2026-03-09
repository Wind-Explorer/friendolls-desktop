import { writable } from "svelte/store";
import { events, type UserStatusPayload } from "$lib/bindings";
import { createEventSource, removeFromStore } from "./listener-utils";

export const friendsPresenceStates = writable<
  Record<string, UserStatusPayload>
>({});
export const currentPresenceState = writable<UserStatusPayload | null>(null);

export const { start: startUserStatus, stop: stopUserStatus } =
  createEventSource(async (addEventListener) => {
    addEventListener(
      await events.friendUserStatusChanged.listen((event) => {
        const { userId, status } = event.payload;

        friendsPresenceStates.update((current) => ({
          ...current,
          [userId]: status,
        }));
      }),
    );

    addEventListener(
      await events.userStatusChanged.listen((event) => {
        currentPresenceState.set(event.payload);
      }),
    );

    addEventListener(
      await events.friendDisconnected.listen((event) => {
        const { userId } = event.payload;
        friendsPresenceStates.update((current) =>
          removeFromStore(current, userId),
        );
      }),
    );
  });
