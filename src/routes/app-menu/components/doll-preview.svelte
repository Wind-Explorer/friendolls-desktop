<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { SPRITE_SETS, SPRITE_SIZE } from "$lib/constants/pet-sprites";
  import { getSpriteSheetUrl } from "$lib/utils/sprite-utils";
  import PetSprite from "$lib/components/PetSprite.svelte";
  import type { DollColorSchemeDto } from "$lib/bindings";

  export let dollColorScheme: DollColorSchemeDto;
  export let spriteScale = 2;
  export let spriteOpacity = 1;

  let previewBase64: string | null = null;
  let error: string | null = null;
  let debounceTimer: number | null = null;
  let currentSprite = { x: -3, y: -3 }; // idle sprite initially
  let currentSetIndex = 0;
  let frameIndex = 0;
  let frameTimer: number;
  let switchTimeout: number;

  const setNames = Object.keys(SPRITE_SETS);

  function generatePreview() {
    error = null;
    getSpriteSheetUrl(dollColorScheme)
      .then((url: string) => {
        previewBase64 = url;
      })
      .catch((e: unknown) => {
        error = (e as Error)?.message ?? String(e);
        console.error(e);
      });
  }

  function debouncedGeneratePreview() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      generatePreview();
    }, 100);
  }

  function updateFrame() {
    const setName = setNames[currentSetIndex];
    const frames = SPRITE_SETS[setName];
    const sprite = frames[frameIndex % frames.length];
    currentSprite = { x: sprite[0] * SPRITE_SIZE, y: sprite[1] * SPRITE_SIZE };
  }

  function switchSet() {
    currentSetIndex = (currentSetIndex + 1) % setNames.length;
    startSet();
  }

  function startSet() {
    const setName = setNames[currentSetIndex];
    const frames = SPRITE_SETS[setName];
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

  $: if (dollColorScheme) {
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

<div
  style="transform: scale({spriteScale}); padding: {spriteScale *
    10}px; opacity: {spriteOpacity};"
>
  <div class="size-8">
    {#if error}
      <div
        class="size-full flex justify-center items-center text-xs text-error"
      >
        Error
      </div>
    {:else if previewBase64}
      <div>
        <PetSprite
          spriteSheetUrl={previewBase64}
          spriteX={currentSprite.x}
          spriteY={currentSprite.y}
          size={32}
        />
      </div>
    {:else}
      <div
        class="size-full skeleton"
        style:background-color={dollColorScheme.body}
      ></div>
    {/if}
  </div>
</div>
