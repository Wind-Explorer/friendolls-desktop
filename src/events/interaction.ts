import { writable } from "svelte/store";
import {
  events,
  type InteractionDeliveryFailedDto,
  type InteractionPayloadDto,
} from "$lib/bindings";
import {
  createMultiListenerSubscription,
  setupHmrCleanup,
} from "./listener-utils";

export const receivedInteractions = writable<Map<string, InteractionPayloadDto>>(
  new Map(),
);

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

const subscription = createMultiListenerSubscription();

/**
 * Starts listening for interaction events (received and delivery failed).
 */
export async function startInteraction() {
  if (subscription.isListening()) return;

  try {
    const unlistenReceived = await events.interactionReceived.listen((event) => {
      addInteraction(event.payload);
    });
    subscription.addUnlisten(unlistenReceived);

    const unlistenFailed = await events.interactionDeliveryFailed.listen(
      (event) => {
        console.error("Interaction delivery failed:", event.payload);
        alert(
          `Failed to send message to user ${event.payload.recipientUserId}: ${event.payload.reason}`,
        );
      },
    );
    subscription.addUnlisten(unlistenFailed);
    subscription.setListening(true);
  } catch (err) {
    console.error("Failed to initialize interaction listeners:", err);
    throw err;
  }
}

export function stopInteraction() {
  subscription.stop();
}

setupHmrCleanup(stopInteraction);
