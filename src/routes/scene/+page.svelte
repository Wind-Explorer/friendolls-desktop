<script lang="ts">
  import { cursorPositionOnScreen } from "../../events/cursor";
  import { friendsCursorPositions } from "../../events/friend-cursor";
  import { appData } from "../../events/app-data";
  import { activeDollSpriteUrl } from "../../events/active-doll-sprite";
  import { friendActiveDollSpriteUrls } from "../../events/friend-active-doll-sprite";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsPresenceStates,
    currentPresenceState,
  } from "../../events/user-status";
  import { commands } from "$lib/bindings";
  import DebugBar from "./components/debug-bar.svelte";
  import Neko from "./components/neko/neko.svelte";
  import PetMenu from "./components/pet-menu/pet-menu.svelte";
  import { createPetActions } from "./components/pet-menu/events";
  import type { UserBasicDto } from "$lib/bindings";

  function getFriend(friendId: string): UserBasicDto | undefined {
    return (
      ($appData?.friends ?? []).find((friend) => friend.friend?.id === friendId)
        ?.friend ?? undefined
    );
  }
</script>

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <button
    class="absolute inset-0 z-10 size-full"
    aria-label="Deactive scene interactive"
    onmousedown={async () => {
      await commands.setSceneInteractive(false, true);
    }}>&nbsp;</button
  >
  {#if $appData?.user?.activeDollId}
    <Neko
      targetX={$cursorPositionOnScreen.raw.x}
      targetY={$cursorPositionOnScreen.raw.y}
      spriteUrl={$activeDollSpriteUrl}
    />
  {/if}
  {#each Object.entries($friendsCursorPositions) as [friendId, position] (friendId)}
    {#if $friendActiveDollSpriteUrls[friendId]}
      {@const friend = getFriend(friendId)}
      <Neko
        targetX={position.raw.x}
        targetY={position.raw.y}
        spriteUrl={$friendActiveDollSpriteUrls[friendId]}
        initialX={position.raw.x}
        initialY={position.raw.y}
      >
        <PetMenu
          actions={createPetActions(friend!)}
          ariaLabel={`Open ${friend?.name} actions`}
        />
      </Neko>
    {/if}
  {/each}
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
