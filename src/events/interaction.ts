import { listen } from "@tauri-apps/api/event";
import { addInteraction } from "$lib/stores/interaction-store";
import type { InteractionPayloadDto } from "../types/bindings/InteractionPayloadDto";
import type { InteractionDeliveryFailedDto } from "../types/bindings/InteractionDeliveryFailedDto";
import { AppEvents } from "../types/bindings/AppEventsConstants";
import {
  createMultiListenerSubscription,
  setupHmrCleanup,
} from "./listener-utils";

const subscription = createMultiListenerSubscription();

export async function initInteractionListeners() {
  if (subscription.isListening()) return;

  const unlistenReceived = await listen<InteractionPayloadDto>(
    AppEvents.InteractionReceived,
    (event) => {
      addInteraction(event.payload);
    },
  );
  subscription.addUnlisten(unlistenReceived);

  const unlistenFailed = await listen<InteractionDeliveryFailedDto>(
    AppEvents.InteractionDeliveryFailed,
    (event) => {
      console.error("Interaction delivery failed:", event.payload);
      // You might want to show a toast or alert here
      alert(
        `Failed to send message to user ${event.payload.recipientUserId}: ${event.payload.reason}`,
      );
    },
  );
  subscription.addUnlisten(unlistenFailed);
  subscription.setListening(true);
}

export function stopInteractionListeners() {
  subscription.stop();
}

setupHmrCleanup(stopInteractionListeners);
