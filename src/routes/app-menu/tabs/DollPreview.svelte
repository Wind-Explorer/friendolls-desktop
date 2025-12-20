<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let bodyColor: string;
  export let outlineColor: string;
  export let applyTexture: boolean = true;

  let previewBase64: string | null = null;
  let error: string | null = null;
  let debounceTimer: number | null = null;
  let currentSprite = { x: -3, y: -3 }; // idle sprite initially
  let currentSetIndex = 0;
  let frameIndex = 0;
  let frameTimer: number;
  let switchTimeout: number;

  // Sprite constants from DesktopPet.svelte
  const spriteSets: Record<string, [number, number][]> = {
    idle: [[-3, -3]],
    alert: [[-7, -3]],
    scratchSelf: [
      [-5, 0],
      [-6, 0],
      [-7, 0],
    ],
    scratchWallN: [
      [0, 0],
      [0, -1],
    ],
    scratchWallS: [
      [-7, -1],
      [-6, -2],
    ],
    scratchWallE: [
      [-2, -2],
      [-2, -3],
    ],
    scratchWallW: [
      [-4, 0],
      [-4, -1],
    ],
    tired: [[-3, -2]],
    sleeping: [
      [-2, 0],
      [-2, -1],
    ],
    N: [
      [-1, -2],
      [-1, -3],
    ],
    NE: [
      [0, -2],
      [0, -3],
    ],
    E: [
      [-3, 0],
      [-3, -1],
    ],
    SE: [
      [-5, -1],
      [-5, -2],
    ],
    S: [
      [-6, -3],
      [-7, -2],
    ],
    SW: [
      [-5, -3],
      [-6, -1],
    ],
    W: [
      [-4, -2],
      [-4, -3],
    ],
    NW: [
      [-1, 0],
      [-1, -1],
    ],
  };

  const setNames = Object.keys(spriteSets);

  function generatePreview() {
    error = null;
    try {
      invoke<string>("recolor_gif_base64", {
        whiteColorHex: bodyColor,
        blackColorHex: outlineColor,
        applyTexture,
      })
        .then((result) => {
          previewBase64 = result;
        })
        .catch((e) => {
          error = (e as Error)?.message ?? String(e);
          console.error(e);
        });
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error(e);
    }
  }

  function debouncedGeneratePreview() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      generatePreview();
    }, 300); // Adjust debounce delay as needed (300ms is a common starting point)
  }

  function updateFrame() {
    const setName = setNames[currentSetIndex];
    const frames = spriteSets[setName];
    const sprite = frames[frameIndex % frames.length];
    currentSprite = { x: sprite[0] * 32, y: sprite[1] * 32 };
  }

  function switchSet() {
    currentSetIndex = (currentSetIndex + 1) % setNames.length;
    startSet();
  }

  function startSet() {
    const setName = setNames[currentSetIndex];
    const frames = spriteSets[setName];
    frameIndex = 0;
    updateFrame();
    const frameDuration = frames.length > 0 ? 1000 / frames.length : 1000;
    frameTimer = setInterval(() => {
      frameIndex++;
      updateFrame();
    }, frameDuration);
    switchTimeout = setTimeout(() => {
      clearInterval(frameTimer);
      switchSet();
    }, 3000);
  }

  $: if (bodyColor && outlineColor) {
    debouncedGeneratePreview();
  }

  onMount(() => {
    startSet();
  });

  onDestroy(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (frameTimer) clearInterval(frameTimer);
    if (switchTimeout) clearTimeout(switchTimeout);
  });
</script>

<div class="scale-150 p-2">
  <div class="size-8">
    {#if error}
      <div
        class="size-full flex justify-center items-center text-xs text-error"
      >
        Error
      </div>
    {:else if previewBase64}
      <div
        class="pixelated"
        style="
          width: 32px;
          height: 32px;
          background-image: url('data:image/gif;base64,{previewBase64}');
          background-position: {currentSprite.x}px {currentSprite.y}px;
        "
      ></div>
    {:else}
      <div class="size-full skeleton" style:background-color={bodyColor}></div>
    {/if}
  </div>
</div>
