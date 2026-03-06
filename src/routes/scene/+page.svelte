<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsUserStatuses,
    currentUserStatus,
  } from "../../events/user-status";
  import { invoke } from "@tauri-apps/api/core";
  import DebugBar from "./components/debug-bar.svelte";
</script>

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
  <div id="debug-bar">
    <DebugBar
      isInteractive={$sceneInteractive}
      cursorPosition={$cursorPositionOnScreen}
      presenceStatus={$currentUserStatus?.presenceStatus ?? null}
      friendsCursorPositions={$friendsCursorPositions}
      friends={$appData?.friends ?? []}
      friendsUserStatuses={$friendsUserStatuses}
    />
  </div>
</div>
