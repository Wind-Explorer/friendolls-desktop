<script>
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";
  import { initCursorTracking, stopCursorTracking } from "../events/cursor";
  import { initAppDataListener } from "../events/app-data";

  let { children } = $props();
  if (browser) {
    onMount(async () => {
      try {
        await initCursorTracking();
        await initAppDataListener();
      } catch (err) {
        console.error("[Scene] Failed to initialize event listeners:", err);
      }
    });

    onDestroy(() => {
      stopCursorTracking();
    });
  }
</script>

<div class="size-full bg-transparent">
  {@render children?.()}
</div>
