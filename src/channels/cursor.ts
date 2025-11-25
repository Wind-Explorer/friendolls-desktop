import { Channel, invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export type CursorPositions = {
  raw: { x: number; y: number };
  mapped: { x: number; y: number };
};
export let cursorPositionOnScreen = writable<CursorPositions>({
  raw: { x: 0, y: 0 },
  mapped: { x: 0, y: 0 },
});

export function initChannelCursorPosition() {
  const channel = new Channel<CursorPositions>();
  channel.onmessage = (pos) => {
    cursorPositionOnScreen.set(pos);
  };
  invoke("channel_cursor_positions", { onEvent: channel });
}
