<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsUserStatuses,
    type UserStatus,
  } from "../../events/user-status";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { AppEvents } from "../../types/bindings/AppEventsConstants";
  import { onMount } from "svelte";
  import type { PresenceStatus } from "../../types/bindings/PresenceStatus";
  import DebugBar from "./components/debug-bar.svelte";

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  let isInteractive = $derived($sceneInteractive);

  let presenceStatus: PresenceStatus | null = $state(null);

  onMount(() => {
    const unlisten = listen<UserStatus>(
      AppEvents.UserStatusChanged,
      (event) => {
        presenceStatus = event.payload.presenceStatus;
      },
    );

    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <button
    class="absolute inset-0 z-10 size-full"
    aria-label="Deactive scene interactive"
    onmousedown={async () => {
      await invoke("set_scene_interactive", {
        interactive: false,
        shouldClick: true,
      });
    }}>&nbsp;</button
  >
  <div
    id="debug-bar"
  >
    <DebugBar
      {isInteractive}
      cursorPosition={$cursorPositionOnScreen}
      {presenceStatus}
      friendsCursorPositions={$friendsCursorPositions}
      friends={$appData?.friends ?? []}
      friendsUserStatuses={$friendsUserStatuses}
    />
  </div>
</div>
