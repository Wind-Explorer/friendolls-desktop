import { writable } from "svelte/store";
import type { InteractionPayloadDto } from "../../types/bindings/InteractionPayloadDto";

// Map senderUserId -> InteractionPayloadDto
export const receivedInteractions = writable<Map<string, InteractionPayloadDto>>(new Map());

export function addInteraction(interaction: InteractionPayloadDto) {
  receivedInteractions.update((map) => {
    // For now, we only store the latest message per user.
    // In the future, we could store an array if we want a history.
    const newMap = new Map(map);
    newMap.set(interaction.senderUserId, interaction);
    return newMap;
  });
}

export function clearInteraction(userId: string) {
  receivedInteractions.update((map) => {
    const newMap = new Map(map);
    newMap.delete(userId);
    return newMap;
  });
}
