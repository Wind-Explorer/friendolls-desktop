<script lang="ts">
  import Friends from "./tabs/friends.svelte";
  import Preferences from "./tabs/preferences.svelte";
  import YourDolls from "./tabs/your-dolls.svelte";
</script>

<div
  class="p-2 h-full absolute inset-0 bg-base-100 border-base-200/50 border border-t-0 rounded-b-xl"
>
  <div class="flex flex-col gap-2 h-full">
    <div class="size-full">
      <div class="tabs tabs-lift h-full">
        <input
          type="radio"
          name="app_menu_tabs"
          class="tab"
          aria-label="Your Dolls"
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
