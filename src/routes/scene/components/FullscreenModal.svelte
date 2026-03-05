<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { type TransitionConfig } from "svelte/transition";
  import PetSprite from "$lib/components/PetSprite.svelte";
  import { SPRITE_SETS, SPRITE_SIZE } from "$lib/constants/pet-sprites";

  function fadeSlide(
    node: HTMLElement,
    params: { duration: number }
  ): TransitionConfig {
    const opacity = parseFloat(getComputedStyle(node).opacity);
    return {
      duration: params.duration,
      easing: cubicOut,
      css: (t) => `opacity: ${t * opacity}; transform: translateY(${(1 - t) * 20}px);`,
    };
  }

  const idleSprite = {
    x: SPRITE_SETS.idle[0][0] * SPRITE_SIZE,
    y: SPRITE_SETS.idle[0][1] * SPRITE_SIZE,
  };

  let {
    imageSrc,
    senderSpriteUrl = "",
    visible = $bindable(false),
    senderName = "",
  }: {
    imageSrc: string;
    senderSpriteUrl?: string;
    visible: boolean;
    senderName?: string;
  } = $props();

</script>

{#if visible}
  <div
    class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-gradient-to-b from-transparent to-black/80"
    transition:fadeSlide={{ duration: 300 }}
  >
    {#if senderName}
      <div class="mb-4 text-white text-lg font-medium">{senderName} gave you a headpat!</div>
    {/if}
    <div class="flex items-center justify-center gap-6">
      {#if senderSpriteUrl}
        <div class="flex items-center justify-center size-32">
          <div style="transform: scale(4); transform-origin: center;">
            <PetSprite
              spriteSheetUrl={senderSpriteUrl}
              spriteX={idleSprite.x}
              spriteY={idleSprite.y}
              size={32}
            />
          </div>
        </div>
      {/if}
      <img
        src={imageSrc}
        alt="Headpat"
        class="max-w-full max-h-full object-contain"
      />
    </div>
  </div>
{/if}
