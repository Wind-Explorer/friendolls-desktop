<script lang="ts">
  import { onMount } from "svelte";
  import { nekoPositions } from "../../events/neko-positions";
  import { appData } from "../../events/app-data";
  import { activeDollSpriteUrl } from "../../events/active-doll-sprite";
  import { friendActiveDollSpriteUrls } from "../../events/friend-active-doll-sprite";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsPresenceStates,
    currentPresenceState,
  } from "../../events/user-status";
  import { commands, type NekoPositionDto } from "$lib/bindings";
  import DebugBar from "./components/debug-bar.svelte";
  import Neko from "./components/neko/neko.svelte";
  import PetMenu from "./components/pet-menu/pet-menu.svelte";
  import PetMessagePop from "./components/pet-message-pop.svelte";
  import PetMessageSend from "./components/pet-message-send.svelte";
  import type { UserBasicDto } from "$lib/bindings";
  import { appState } from "../../events/app-state";

  let debugMode = $state(false);

  onMount(async () => {
    const config = await commands.getClientConfig();
    debugMode = config.debug_mode;
  });

  function getFriend(friendId: string): UserBasicDto | undefined {
    return (
      ($appData?.friends ?? []).find((friend) => friend.friend?.id === friendId)
        ?.friend ?? undefined
    );
  }

  let nekoEntries = $derived.by(() => {
    return Object.entries($nekoPositions).filter(
      (entry): entry is [string, NekoPositionDto] => entry[1] !== undefined,
    );
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
  {#each nekoEntries as [userId, position] (userId)}
    {@const spriteUrl = position.isSelf
      ? $activeDollSpriteUrl
      : $friendActiveDollSpriteUrls[userId]}
    {#if spriteUrl}
      {@const friend = position.isSelf ? undefined : getFriend(userId)}
      <Neko
        targetX={position.target.x}
        targetY={position.target.y}
        spriteUrl={spriteUrl}
        initialX={position.target.x}
        initialY={position.target.y}
        scale={$appState.sceneSetup.nekosScale}
        opacity={$appState.sceneSetup.nekosOpacity}
      >
        {#if !position.isSelf && friend}
          <PetMenu user={friend} ariaLabel={`Open ${friend.name} actions`} />
          <PetMessagePop userId={userId} />
          <PetMessageSend userId={userId} userName={friend.name} />
        {/if}
      </Neko>
    {/if}
  {/each}
  {#if debugMode}
    <div id="debug-bar">
        <DebugBar
          isInteractive={$sceneInteractive}
          nekoPositions={$nekoPositions}
          presenceStatus={$currentPresenceState?.presenceStatus ?? null}
          friends={$appData?.friends ?? []}
          friendsPresenceStates={$friendsPresenceStates}
        />
    </div>
  {/if}
</div>
