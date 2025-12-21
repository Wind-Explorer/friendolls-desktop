<script lang="ts">
  import Friends from "./tabs/friends.svelte";
  import Preferences from "./tabs/preferences.svelte";
  import YourDolls from "./tabs/your-dolls.svelte";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
</script>

<div
  class="p-2 h-full absolute inset-0 bg-base-100 border-base-200/50 border border-t-0 rounded-b-xl"
>
  <div class="flex flex-col gap-2 h-full">
    <div class="size-full flex flex-col gap-2">
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
      <div class="w-full flex flex-row justify-between">
        <div></div>
        <div class="flex flex-row gap-2">
          <button
            class="btn btn-sm btn-outline border-neutral-500/50"
            onclick={async () => {
              await getCurrentWebviewWindow().close();
            }}><p class="px-4">Ok</p></button
          >
        </div>
      </div>
    </div>
  </div>
</div>
