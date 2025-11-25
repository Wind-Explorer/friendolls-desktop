import { Channel, invoke } from "@tauri-apps/api/core";
import { writable } from "svelte/store";

export let cursorPositionOnScreen = writable<{ x: number; y: number }>({ x: 0, y: 0 });

export function initCursorPositionStream() {
  const channel = new Channel<{ x: number; y: number }>();
  channel.onmessage = (pos) => {
    cursorPositionOnScreen.set(pos);
  };
  invoke("stream_cursor_position", { onEvent: channel });
}