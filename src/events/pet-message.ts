import { writable } from "svelte/store";
import { commands } from "$lib/bindings";

export const openPetMessageSendUserId = writable<string | null>(null);

function menuStateId(userId: string) {
  return `${userId}:pet-message-send`;
}

export function openPetMessageSend(userId: string) {
  openPetMessageSendUserId.update((currentUserId) => {
    if (currentUserId && currentUserId !== userId) {
      void commands.setPetMenuState(menuStateId(currentUserId), false);
    }
    return userId;
  });
  void commands.setPetMenuState(menuStateId(userId), true);
}

export function closePetMessageSend() {
  openPetMessageSendUserId.update((currentUserId) => {
    if (currentUserId) {
      void commands.setPetMenuState(menuStateId(currentUserId), false);
    }
    return null;
  });
}
