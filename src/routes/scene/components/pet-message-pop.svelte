<script lang="ts">
  import { onDestroy } from "svelte";
  import {
    receivedInteractions,
    clearInteraction,
  } from "../../../events/interaction";

  interface Props {
    userId: string;
  }

  let { userId }: Props = $props();

  let visible = $state(false);
  let hideTimer: ReturnType<typeof setTimeout> | null = null;
  let lastTimestamp: string | null = null;

  const interaction = $derived($receivedInteractions.get(userId));

  function clearHideTimer() {
    if (hideTimer) {
      clearTimeout(hideTimer);
      hideTimer = null;
    }
  }

  function dismiss() {
    visible = false;
    clearHideTimer();
    clearInteraction(userId);
  }

  $effect(() => {
    if (!interaction) {
      visible = false;
      lastTimestamp = null;
      clearHideTimer();
      return;
    }

    if (interaction.timestamp !== lastTimestamp) {
      lastTimestamp = interaction.timestamp;
      visible = true;
      clearHideTimer();
      hideTimer = setTimeout(() => {
        dismiss();
      }, 14000);
    }
  });

  onDestroy(() => {
    clearHideTimer();
  });
</script>

<div
  class={`absolute card bottom-9 flex flex-col left-4 z-40 w-max max-w-52 border border-base-300 shadow-lg bg-base-100 px-2 py-1.5 text-base-content transition-all duration-200 ease-out ${
    visible
      ? "pointer-events-auto opacity-100"
      : "pointer-events-none opacity-0"
  }`}
>
  {#if interaction}
    <p class="line-clamp-3 text-xs leading-snug wrap-break-words">
      {interaction.content}
    </p>
  {/if}
</div>
