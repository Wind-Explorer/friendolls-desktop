<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { usePetState } from "$lib/composables/usePetState";
  import { getSpriteSheetUrl } from "$lib/utils/sprite-utils";
  import PetSprite from "$lib/components/PetSprite.svelte";
  import onekoGif from "../../../assets/oneko/oneko.gif";
  import PetMenu from "./PetMenu.svelte";
  import type { DollDto } from "../../../types/bindings/DollDto";
  import type { UserBasicDto } from "../../../types/bindings/UserBasicDto";

  export let id = "";
  export let targetX = 0;
  export let targetY = 0;
  export let user: UserBasicDto;
  export let doll: DollDto | undefined = undefined;
  export let isInteractive = false;

  const { position, currentSprite, updatePosition, setPosition } = usePetState(
    32,
    32,
  );

  let animationFrameId: number;
  let lastFrameTimestamp: number;
  let spriteSheetUrl = onekoGif;

  let isPetMenuOpen = false;

  // Watch for color changes to regenerate sprite
  $: updateSprite(
    doll?.configuration.colorScheme.body,
    doll?.configuration.colorScheme.outline,
  );

  $: (isInteractive, (isPetMenuOpen = false));

  $: {
    if (id) {
      invoke("set_pet_menu_state", { id, open: isPetMenuOpen });
    }
  }

  async function updateSprite(
    body: string | undefined,
    outline: string | undefined,
  ) {
    if (body && outline) {
      spriteSheetUrl = await getSpriteSheetUrl({
        bodyColor: body,
        outlineColor: outline,
      });
    } else {
      spriteSheetUrl = onekoGif;
    }
  }

  function frame(timestamp: number) {
    if (!lastFrameTimestamp) {
      lastFrameTimestamp = timestamp;
    }

    // 100ms per frame for the animation loop
    if (timestamp - lastFrameTimestamp > 100) {
      lastFrameTimestamp = timestamp;
      if (!isPetMenuOpen) {
        updatePosition(targetX, targetY, window.innerWidth, window.innerHeight);
      }
    }

    animationFrameId = requestAnimationFrame(frame);
  }

  onMount(() => {
    // Initialize position to target so it doesn't fly in from 32,32 every time
    setPosition(targetX, targetY);
    animationFrameId = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  });
</script>

<div
  class="desktop-pet flex flex-col items-center relative"
  style="
    transform: translate({$position.x - 16}px, {$position.y - 16}px);
    z-index: 50;
  "
>
  {#if isPetMenuOpen}
    <div
      class="absolute -translate-y-24 w-50 h-22 *:size-full shadow-md rounded"
      role="menu"
      tabindex="0"
      aria-label="Pet Menu"
    >
      {#if doll}
        <PetMenu {doll} {user} />
      {/if}
    </div>
  {/if}
  <button
    onclick={() => {
      isPetMenuOpen = !isPetMenuOpen;
    }}
  >
    <PetSprite
      {spriteSheetUrl}
      spriteX={$currentSprite.x}
      spriteY={$currentSprite.y}
    />
  </button>

  <span
    class="absolute -bottom-5 width-full text-[10px] bg-black/50 text-white px-1 rounded backdrop-blur-sm mt-1 whitespace-nowrap opacity-0 transition-opacity"
    class:opacity-100={isInteractive && !isPetMenuOpen}
  >
    {doll?.name}
  </span>
</div>

<style>
  .desktop-pet {
    position: fixed; /* Fixed relative to the viewport/container */
    top: 0;
    left: 0;
    will-change: transform;
  }
</style>
