<script>
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";
  import { initCursorTracking, stopCursorTracking } from "../events/cursor";
  import { initAppDataListener } from "../events/app-data";
  import { initInteractionListeners, stopInteractionListeners } from "../events/interaction";
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
        await initInteractionListeners();
      } catch (err) {
        console.error("Failed to initialize event listeners:", err);
      }
    });

    onDestroy(() => {
      stopCursorTracking();
      stopSceneInteractiveListener();
      stopInteractionListeners();
    });
  }
</script>

<div class="w-screen h-screen bg-transparent">
  {@render children?.()}
</div>
