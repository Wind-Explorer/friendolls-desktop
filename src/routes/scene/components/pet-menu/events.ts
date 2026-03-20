import type { UserBasicDto } from "$lib/bindings";
import { openPetMessageSend } from "../../../../events/pet-message";

export type CloseHandler = () => void;

export function createPetActions(user: UserBasicDto) {
  return [
    {
      icon: "👋",
      label: `Wave at ${user.name}`,
      onClick: () => {
        console.log(`Wave at ${user.name}`);
      },
    },
    {
      icon: "💬",
      label: `Message ${user.name}`,
      onClick: () => {
        openPetMessageSend(user.id);
      },
    },
    {
      icon: "🔔",
      label: `Ping ${user.name}`,
      onClick: () => {
        console.log(`Ping ${user.name}`);
      },
    },
    {
      icon: "🔎",
      label: `Inspect ${user.name}`,
      onClick: () => {
        console.log(`Inspect ${user.name}`);
      },
    },
  ];
}

export function createDocumentPointerHandler(
  isOpen: () => boolean,
  rootEl: () => HTMLDivElement | null,
  closeMenu: CloseHandler,
) {
  return function handleDocumentPointerDown(event: PointerEvent) {
    if (!isOpen() || !rootEl()) {
      return;
    }

    if (event.target instanceof Node && !rootEl()!.contains(event.target)) {
      closeMenu();
    }
  };
}

export function createKeyDownHandler(
  isOpen: () => boolean,
  closeMenu: CloseHandler,
) {
  return function handleKeyDown(event: KeyboardEvent) {
    if (!isOpen()) {
      return;
    }

    if (event.key === "Escape") {
      closeMenu();
      event.preventDefault();
    }
  };
}
