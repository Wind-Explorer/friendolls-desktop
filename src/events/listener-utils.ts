import type { UnlistenFn } from "@tauri-apps/api/event";

export type ListenerSubscription = {
  stop: () => void;
  isListening: () => boolean;
  setListening: (value: boolean) => void;
  addEventListener: (unlisten: UnlistenFn | null) => void;
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
    addEventListener: (unlisten) => {
      if (unlisten) {
        unlistens.push(unlisten);
      }
    },
  };
}

export type EventSource = {
  start: () => Promise<void>;
  stop: () => void;
  isListening: () => boolean;
};

export function createEventSource(
  setup: (addEventListener: (unlisten: UnlistenFn) => void) => Promise<void>,
  stopFn: () => void = () => {},
): EventSource {
  const subscription = createListenersSubscription(stopFn);

  async function start() {
    if (subscription.isListening()) return;
    try {
      await setup((unlisten) => subscription.addEventListener(unlisten));
      subscription.setListening(true);
    } catch (err) {
      subscription.stop();
      console.error(`Failed to start:`, err);
      throw err;
    }
  }

  function stop() {
    subscription.stop();
  }

  if (import.meta.hot) {
    import.meta.hot.dispose(() => stop());
  }

  return { start, stop, isListening: () => subscription.isListening() };
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
