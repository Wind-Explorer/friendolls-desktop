import type { UnlistenFn } from "@tauri-apps/api/event";

export type ListenerSubscription = {
  stop: () => void;
  isListening: () => boolean;
  setListening: (value: boolean) => void;
  addUnlisten: (unlisten: UnlistenFn | null) => void;
};

export function createListenersSubscription(
  stopFn: () => void = () => {},
): ListenerSubscription {
  let unlistens: UnlistenFn[] = [];
  let listening = false;

  return {
    stop: () => {
      for (const unlisten of unlistens) {
        unlisten();
      }
      unlistens = [];
      listening = false;
      stopFn();
    },
    isListening: () => listening,
    setListening: (value) => {
      listening = value;
    },
    addUnlisten: (unlisten) => {
      if (unlisten) {
        unlistens.push(unlisten);
      }
    },
  };
}

export function setupHmrCleanup(cleanup: () => void) {
  if (import.meta.hot) {
    import.meta.hot.dispose(() => {
      cleanup();
    });
  }
}

export function removeFromStore<T>(
  current: Record<string, T>,
  key: string,
): Record<string, T> {
  if (!(key in current)) return current;
  const next = { ...current };
  delete next[key];
  return next;
}
