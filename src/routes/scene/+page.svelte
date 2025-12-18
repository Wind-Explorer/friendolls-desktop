<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";

  import DesktopPet from "./DesktopPet.svelte";

  let innerWidth = 0;
  let innerHeight = 0;

  function getFriendName(userId: string) {
    const friend = $appData?.friends?.find((f) => f.friend.id === userId);
    return friend ? friend.friend.name : userId.slice(0, 8) + "...";
  }
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <div
    class="size-max mx-auto bg-base-100 border-base-200 border px-4 py-3 rounded-xl"
  >
    <div class="flex flex-col text-center">
      <p class="text-xl">Friendolls</p>
      <p class="text-sm opacity-50">Scene Screen</p>
      <div class="mt-4 flex flex-col gap-1">
        <span class="font-mono text-sm">
          Raw: ({$cursorPositionOnScreen.raw.x}, {$cursorPositionOnScreen.raw
            .y})
        </span>
        <span class="font-mono text-sm">
          Mapped: ({$cursorPositionOnScreen.mapped.x.toFixed(3)}, {$cursorPositionOnScreen.mapped.y.toFixed(
            3,
          )})
        </span>
      </div>

      {#if Object.keys($friendsCursorPositions).length > 0}
        <div class="mt-4 flex flex-col gap-2">
          <p class="text-sm font-semibold opacity-75">Friends Online</p>
          <div>
            {#each Object.entries($friendsCursorPositions) as [userId, position]}
              <div
                class="bg-base-200/50 p-2 rounded text-xs text-left flex gap-2 flex-col"
              >
                <span class="font-bold">{getFriendName(userId)}</span>
                <div class="flex flex-col font-mono">
                  <span>
                    Raw: ({position.raw.x}, {position.raw.y})
                  </span>
                  <span>
                    Mapped: ({position.mapped.x.toFixed(3)}, {position.mapped.y.toFixed(
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
        <DesktopPet
          targetX={position.mapped.x * innerWidth}
          targetY={position.mapped.y * innerHeight}
          name={getFriendName(userId)}
        />
      {/each}
    {/if}
  </div>
</div>
