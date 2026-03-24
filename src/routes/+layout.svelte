<script>
  import { browser } from "$app/environment";
  import { onMount, onDestroy } from "svelte";
  import { startCursorTracking, stopCursorTracking } from "../events/cursor";
  import {
    startFriendCursorTracking,
    stopFriendCursorTracking,
  } from "../events/friend-cursor";
  import {
    startActiveDollSprite,
    stopActiveDollSprite,
  } from "../events/active-doll-sprite";
  import {
    startFriendActiveDollSprite,
    stopFriendActiveDollSprite,
  } from "../events/friend-active-doll-sprite";
  import { startAppData } from "../events/app-data";
  import { startAppState, stopAppState } from "../events/app-state";
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
        await startAppState();
        await startActiveDollSprite();
        await startFriendActiveDollSprite();
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
      stopActiveDollSprite();
      stopFriendActiveDollSprite();
      stopSceneInteractive();
      stopInteraction();
      stopUserStatus();
      stopAppState();
    });
  }
</script>

<div class="w-screen h-screen bg-transparent">
  {@render children?.()}
</div>
