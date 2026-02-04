<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
    friendsActiveDolls,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";
  import { friendsUserStatuses } from "../../events/user-status";
  import { invoke } from "@tauri-apps/api/core";
  import DesktopPet from "./components/DesktopPet.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import type { AppMetadata } from "../../types/bindings/AppMetadata";

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  let isInteractive = $derived($sceneInteractive);

  function getFriendById(userId: string) {
    const friend = $appData?.friends?.find((f) => f.friend.id === userId);
    return friend!.friend;
  }

  function getFriendDoll(userId: string) {
    if (userId in $friendsActiveDolls) {
      return $friendsActiveDolls[userId];
    }

    const friend = $appData?.friends?.find((f) => f.friend.id === userId);
    return friend?.friend.activeDoll;
  }

  function getFriendStatus(userId: string) {
    return $friendsUserStatuses[userId];
  }

  let appMetadata: AppMetadata | null = $state(null);

  onMount(() => {
    const unlisten = listen<AppMetadata>("active-app-changed", (event) => {
      appMetadata = event.payload;
    });

    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <button
    class="absolute inset-0 z-10 size-full"
    aria-label="Deactive scene interactive"
    onmousedown={async () => {
      await invoke("set_scene_interactive", {
        interactive: false,
        shouldClick: true,
      });
    }}>&nbsp;</button
  >
  <div
    class="size-max mx-auto bg-base-100 border-base-200 border p-1 rounded-lg shadow-md"
  >
    <div class="flex flex-row gap-1 items-center text-center">
      <div>
        <span class="py-3 text-xs items-center gap-2 badge">
          <span
            class={`size-2 rounded-full ${isInteractive ? "bg-success" : "bg-base-300"}`}
          ></span>
          Intercepting cursor events
        </span>
      </div>

      <span class="font-mono text-xs badge py-3">
        ({$cursorPositionOnScreen.mapped.x.toFixed(3)}, {$cursorPositionOnScreen.mapped.y.toFixed(
          3,
        )})
      </span>

      <span class="font-mono text-xs badge py-3 flex items-center gap-2">
        {#if appMetadata?.appIconB64}
          <img
            src={`data:image/png;base64,${appMetadata.appIconB64}`}
            alt="Active app icon"
            class="size-4"
          />
        {/if}
        {appMetadata?.localized}
      </span>

      {#if Object.keys($friendsCursorPositions).length > 0}
        <div class="flex flex-col gap-2">
          <div>
            {#each Object.entries($friendsCursorPositions) as [userId, position]}
              {@const status = getFriendStatus(userId)}
              <div class="badge py-3 text-xs text-left flex flex-row gap-2">
                <span class="font-bold">{getFriendById(userId).name}</span>
                <div class="flex flex-row font-mono gap-2">
                  <span>
                    ({position.mapped.x.toFixed(3)}, {position.mapped.y.toFixed(
                      3,
                    )})
                  </span>
                  {#if status}
                    <span class="flex items-center gap-1">
                      {status.state} in
                      {#if status.appMetadata.appIconB64}
                        <img
                          src={`data:image/png;base64,${status.appMetadata.appIconB64}`}
                          alt="Friend's active app icon"
                          class="size-4"
                        />
                      {/if}
                      {status.appMetadata.localized ||
                        status.appMetadata.unlocalized}
                    </span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="absolute inset-0 size-full">
    {#if Object.keys($friendsCursorPositions).length > 0}
      {#each Object.entries($friendsCursorPositions) as [userId, position]}
        {@const doll = getFriendDoll(userId)}
        {#if doll}
          <DesktopPet
            id={userId}
            targetX={position.mapped.x * innerWidth}
            targetY={position.mapped.y * innerHeight}
            user={getFriendById(userId)}
            {doll}
            {isInteractive}
          />
        {/if}
      {/each}
    {/if}
  </div>
</div>
