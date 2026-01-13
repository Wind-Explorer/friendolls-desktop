import { listen } from "@tauri-apps/api/event";
import { addInteraction } from "$lib/stores/interaction-store";
import type { InteractionPayloadDto } from "../types/bindings/InteractionPayloadDto";
import type { InteractionDeliveryFailedDto } from "../types/bindings/InteractionDeliveryFailedDto";

let unlistenReceived: (() => void) | undefined;
let unlistenFailed: (() => void) | undefined;

export async function initInteractionListeners() {
  unlistenReceived = await listen<InteractionPayloadDto>(
    "interaction-received",
    (event) => {
      console.log("Received interaction:", event.payload);
      addInteraction(event.payload);
    },
  );

  unlistenFailed = await listen<InteractionDeliveryFailedDto>(
    "interaction-delivery-failed",
    (event) => {
      console.error("Interaction delivery failed:", event.payload);
      // You might want to show a toast or alert here
      alert(
        `Failed to send message to user ${event.payload.recipientUserId}: ${event.payload.reason}`,
      );
    },
  );
}

export function stopInteractionListeners() {
  if (unlistenReceived) unlistenReceived();
  if (unlistenFailed) unlistenFailed();
}
