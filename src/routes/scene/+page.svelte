<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
    friendsActiveDolls,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";

  import { invoke } from "@tauri-apps/api/core";

  import DesktopPet from "./components/DesktopPet.svelte";

  let innerWidth = 0;
  let innerHeight = 0;

  $: isInteractive = $sceneInteractive;

  function getFriendName(userId: string) {
    const friend = $appData?.friends?.find((f) => f.friend.id === userId);
    return friend ? friend.friend.name : userId.slice(0, 8) + "...";
  }

  function getFriendDoll(userId: string) {
    if (userId in $friendsActiveDolls) {
      return $friendsActiveDolls[userId];
    }

    const friend = $appData?.friends?.find((f) => f.friend.id === userId);
    return friend?.friend.activeDoll;
  }
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

      {#if Object.keys($friendsCursorPositions).length > 0}
        <div class="flex flex-col gap-2">
          <div>
            {#each Object.entries($friendsCursorPositions) as [userId, position]}
              {@const dollConfig = getFriendDoll(userId)}
              <div class="badge py-3 text-xs text-left flex flex-row gap-2">
                <span class="font-bold">{getFriendName(userId)}</span>
                <div class="flex flex-col font-mono">
                  <span>
                    ({position.mapped.x.toFixed(3)}, {position.mapped.y.toFixed(
                      3,
                    )})
                  </span>
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
            name={getFriendName(userId)}
            {doll}
            {isInteractive}
          />
        {/if}
      {/each}
    {/if}
  </div>
</div>
