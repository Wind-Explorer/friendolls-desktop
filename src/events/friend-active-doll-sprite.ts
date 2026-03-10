import { writable } from "svelte/store";
import {
  commands,
  events,
  type FriendActiveDollSpritesDto,
} from "$lib/bindings";
import { createEventSource } from "./listener-utils";

export const friendActiveDollSpriteUrls = writable<Record<string, string>>({});

function toSpriteUrls(
  spriteBase64ByFriendId: FriendActiveDollSpritesDto,
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(spriteBase64ByFriendId)
      .filter((entry): entry is [string, string] => entry[1] !== undefined)
      .map(([friendId, spriteBase64]) => [
        friendId,
        `data:image/gif;base64,${spriteBase64}`,
      ]),
  );
}

export const {
  start: startFriendActiveDollSprite,
  stop: stopFriendActiveDollSprite,
} = createEventSource(async (addEventListener) => {
  const initialSprites = await commands.getFriendActiveDollSpritesBase64();
  friendActiveDollSpriteUrls.set(toSpriteUrls(initialSprites));

  addEventListener(
    await events.friendActiveDollSpritesUpdated.listen((event) => {
      friendActiveDollSpriteUrls.set(toSpriteUrls(event.payload));
    }),
  );
});
