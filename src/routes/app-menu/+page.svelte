<script lang="ts">
  import Friends from "./tabs/friends.svelte";
  import Preferences from "./tabs/preferences.svelte";
  import Modules from "./tabs/modules.svelte";
  import YourDolls from "./tabs/your-dolls/index.svelte";
  import { events } from "$lib/bindings";
  import { onMount } from "svelte";
  import PawPrint from "../../assets/icons/paw-print.svelte";
  import Users from "../../assets/icons/users.svelte";
  import Settings from "../../assets/icons/settings.svelte";
  import Blocks from "../../assets/icons/blocks.svelte";
  import Image from "../../assets/icons/image.svelte";
  import Scene from "./tabs/scene/scene.svelte";

  let showInteractionOverlay = false;

  onMount(() => {
    const unlisten = events.setInteractionOverlay.listen((event) => {
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
        <label class="tab">
          <input
            type="radio"
            name="app_menu_tabs"
            aria-label="Scene Configuration"
            checked
          />
          <div class="*:size-4">
            <Image />
          </div>
        </label>
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Scene />
        </div>

        <label class="tab">
          <input type="radio" name="app_menu_tabs" aria-label="Your Nekos" />
          <div class="*:size-4">
            <PawPrint />
          </div>
        </label>
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <YourDolls />
        </div>

        <label class="tab">
          <input type="radio" name="app_menu_tabs" aria-label="Friends" />
          <div class="*:size-4">
            <Users />
          </div>
        </label>
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Friends />
        </div>

        <label class="tab">
          <input type="radio" name="app_menu_tabs" aria-label="Modules" />
          <div class="*:size-4">
            <Blocks />
          </div>
        </label>
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Modules />
        </div>

        <label class="tab">
          <input type="radio" name="app_menu_tabs" aria-label="Preferences" />
          <div class="*:size-4">
            <Settings />
          </div>
        </label>
        <div class="tab-content bg-base-100 border-base-300 p-4">
          <Preferences />
        </div>
      </div>
    </div>
  </div>
</div>
