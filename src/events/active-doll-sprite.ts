import { writable } from "svelte/store";
import onekoGif from "../assets/oneko/oneko.gif";
import { commands, events } from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const activeDollSpriteUrl = writable(onekoGif);

function toSpriteUrl(spriteBase64: string | null): string {
  return spriteBase64 ? `data:image/gif;base64,${spriteBase64}` : onekoGif;
}

export const {
  start: startActiveDollSprite,
  stop: stopActiveDollSprite,
} = createEventSource(async (addEventListener) => {
  activeDollSpriteUrl.set(
    toSpriteUrl(await commands.getActiveDollSpriteBase64()),
  );

  addEventListener(
    await events.activeDollSpriteChanged.listen((event) => {
      activeDollSpriteUrl.set(toSpriteUrl(event.payload));
    }),
  );
});
