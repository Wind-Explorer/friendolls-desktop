<script lang="ts">
  import { cubicOut } from "svelte/easing";
  import { type TransitionConfig } from "svelte/transition";

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

  let {
    imageSrc,
    visible = $bindable(false),
    senderName = "",
  }: { imageSrc: string; visible: boolean; senderName?: string } = $props();

</script>

{#if visible}
  <div
    class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-gradient-to-b from-transparent to-black/80"
    transition:fadeSlide={{ duration: 300 }}
  >
    {#if senderName}
      <div class="mb-4 text-white text-lg font-medium">{senderName} gave you a headpat!</div>
    {/if}
    <img src={imageSrc} alt="Headpat" class="max-w-full max-h-full object-contain" />
  </div>
{/if}
