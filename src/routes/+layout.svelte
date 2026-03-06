<script>
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";
  import { startCursorTracking, stopCursorTracking } from "../events/cursor";
  import {
    startFriendCursorTracking,
    stopFriendCursorTracking,
  } from "../events/friend-cursor";
  import { startAppData } from "../events/app-data";
  import { startInteraction, stopInteraction } from "../events/interaction";
  import {
    startSceneInteractive,
    stopSceneInteractive,
  } from "../events/scene-interactive";
  import { startUserStatus, stopUserStatus } from "../events/user-status";

  let { children } = $props();
  if (browser) {
    onMount(async () => {
      try {
        await startAppData();
        await startCursorTracking();
        await startFriendCursorTracking();
        await startSceneInteractive();
        await startInteraction();
        await startUserStatus();
      } catch (err) {
        console.error("Failed to initialize event listeners:", err);
      }
    });

    onDestroy(() => {
      stopCursorTracking();
      stopFriendCursorTracking();
      stopSceneInteractive();
      stopInteraction();
      stopUserStatus();
    });
  }
</script>

<div class="w-screen h-screen bg-transparent">
  {@render children?.()}
</div>
