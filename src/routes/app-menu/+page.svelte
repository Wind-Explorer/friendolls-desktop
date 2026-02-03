<script lang="ts">
  import Friends from "./tabs/friends.svelte";
  import Preferences from "./tabs/preferences.svelte";
  import YourDolls from "./tabs/your-dolls/index.svelte";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let showInteractionOverlay = false;

  onMount(() => {
    const unlisten = listen("set-interaction-overlay", (event) => {
      showInteractionOverlay = event.payload as boolean;
    });

    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

<div class="p-2 size-full max-h-full bg-base-100 border-base-200/50">
  {#if showInteractionOverlay}
    <div
      class="absolute inset-0 z-50 cursor-not-allowed rounded-b-xl"
      role="none"
      onclick={(e) => {
        e.stopPropagation();
        e.preventDefault();
      }}
      onkeydown={(e) => {
        e.stopPropagation();
        e.preventDefault();
      }}
      tabindex="-1"
    ></div>
  {/if}
  <div class="flex flex-col gap-2 h-full max-h-full">
    <div class="size-full flex flex-col max-h-full gap-2 h-full">
      <div class="tabs tabs-lift h-full flex-1">
        <input
          type="radio"
          name="app_menu_tabs"
          class="tab"
          aria-label="Your Nekos"
          checked
        />
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <YourDolls />
        </div>

        <input
          type="radio"
          name="app_menu_tabs"
          class="tab"
          aria-label="Friends"
        />
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Friends />
        </div>

        <input
          type="radio"
          name="app_menu_tabs"
          class="tab"
          aria-label="Preferences"
        />
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Preferences />
        </div>
      </div>
    </div>
  </div>
</div>
