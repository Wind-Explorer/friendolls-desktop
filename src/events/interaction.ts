import { writable } from "svelte/store";
import { events, type InteractionPayloadDto } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const receivedInteractions = writable<
  Map<string, InteractionPayloadDto>
>(new Map());

export function addInteraction(interaction: InteractionPayloadDto) {
  receivedInteractions.update((map) => {
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

export const { start: startInteraction, stop: stopInteraction } =
  createEventSource(async (addEventListener) => {
    addEventListener(
      await events.interactionReceived.listen((event) => {
        addInteraction(event.payload);
      }),
    );

    addEventListener(
      await events.interactionDeliveryFailed.listen((event) => {
        console.error("Interaction delivery failed:", event.payload);
        alert(
          `Failed to send message to user ${event.payload.recipientUserId}: ${event.payload.reason}`,
        );
      }),
    );
  });
