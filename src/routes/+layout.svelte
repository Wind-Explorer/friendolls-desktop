<script>
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";
  import { initCursorTracking, stopCursorTracking } from "../events/cursor";
  import { initAppDataListener } from "../events/app-data";
  import {
    initSceneInteractiveListener,
    stopSceneInteractiveListener,
  } from "../events/scene-interactive";

  let { children } = $props();
  if (browser) {
    onMount(async () => {
      try {
        await initCursorTracking();
        await initAppDataListener();
        await initSceneInteractiveListener();
      } catch (err) {
        console.error("Failed to initialize event listeners:", err);
      }
    });

    onDestroy(() => {
      stopCursorTracking();
      stopSceneInteractiveListener();
    });
  }
</script>

<div class="w-screen h-screen bg-transparent">
  {@render children?.()}
</div>
