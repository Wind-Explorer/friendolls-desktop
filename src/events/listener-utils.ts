import type { UnlistenFn } from "@tauri-apps/api/event";

export type ListenerSubscription = {
  stop: () => void;
  isListening: () => boolean;
  setListening: (value: boolean) => void;
  setUnlisten: (unlisten: UnlistenFn | null) => void;
};

export type MultiListenerSubscription = {
  stop: () => void;
  isListening: () => boolean;
  setListening: (value: boolean) => void;
  addUnlisten: (unlisten: UnlistenFn | null) => void;
};

export function createListenerSubscription(
  stopFn: () => void = () => {},
): ListenerSubscription {
  let unlisten: UnlistenFn | null = null;
  let listening = false;

  return {
    stop: () => {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
      listening = false;
      stopFn();
    },
    isListening: () => listening,
    setListening: (value) => {
      listening = value;
    },
    setUnlisten: (next) => {
      unlisten = next;
    },
  };
}

export function createMultiListenerSubscription(
  stopFn: () => void = () => {},
): MultiListenerSubscription {
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

export function parseEventPayload<T>(
  payload: unknown,
  errorLabel: string,
): T | null {
  if (typeof payload === "string") {
    try {
      return JSON.parse(payload) as T;
    } catch (error) {
      console.error(`Failed to parse ${errorLabel} payload`, error);
      return null;
    }
  }

  return payload as T;
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
