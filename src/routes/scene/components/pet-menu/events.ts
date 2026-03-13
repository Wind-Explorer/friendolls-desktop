export type CloseHandler = () => void;

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
