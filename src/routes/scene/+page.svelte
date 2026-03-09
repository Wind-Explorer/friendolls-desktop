<script lang="ts">
  import { cursorPositionOnScreen } from "../../events/cursor";
  import { friendsCursorPositions } from "../../events/friend-cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsPresenceStates,
    currentPresenceState,
  } from "../../events/user-status";
  import { commands } from "$lib/bindings";
  import { getSpriteSheetUrl } from "$lib/utils/sprite-utils";
  import DebugBar from "./components/debug-bar.svelte";
  import Neko from "./components/neko/neko.svelte";

  let spriteUrl = $state("");

  $effect(() => {
    $appData;
    if (!$appData) return;
    commands.getActiveDollColorScheme().then((colorScheme) => {
      getSpriteSheetUrl(colorScheme ?? undefined).then((url) => {
        spriteUrl = url;
      });
    });
  });
</script>

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <button
    class="absolute inset-0 z-10 size-full"
    aria-label="Deactive scene interactive"
    onmousedown={async () => {
      await commands.setSceneInteractive(false, true);
    }}>&nbsp;</button
  >
  <Neko
    targetX={$cursorPositionOnScreen.raw.x}
    targetY={$cursorPositionOnScreen.raw.y}
    {spriteUrl}
  />
  <div id="debug-bar">
    <DebugBar
      isInteractive={$sceneInteractive}
      cursorPosition={$cursorPositionOnScreen}
      presenceStatus={$currentPresenceState?.presenceStatus ?? null}
      friendsCursorPositions={$friendsCursorPositions}
      friends={$appData?.friends ?? []}
      friendsPresenceStates={$friendsPresenceStates}
    />
  </div>
</div>
