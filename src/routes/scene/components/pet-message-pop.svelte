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
      }, 4000);
    }
  });

  onDestroy(() => {
    clearHideTimer();
  });
</script>

<div
  class={`absolute card bottom-9 flex flex-col left-4 z-40 w-52 border border-base-300 bg-base-100 p-2 text-base-content transition-all duration-200 ease-out ${
    visible
      ? "pointer-events-auto opacity-100"
      : "pointer-events-none opacity-0"
  }`}
>
  {#if interaction}
    <div class="flex items-start justify-between gap-2">
      <p
        class="truncate text-[8px] font-semibold uppercase tracking-wide opacity-70"
      >
        {interaction.senderName}
      </p>
    </div>
    <p class="line-clamp-3 text-sm leading-snug mt-1 wrap-break-words">
      {interaction.content}
    </p>
  {/if}
</div>
