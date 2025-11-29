<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import {
    cursorPositionOnScreen,
  } from "../../events/cursor";

</script>

<div class="p-4">
  <div class="flex flex-col gap-4">
    <div>
      <p class="text-xl font-semibold">Preferences</p>
      <p class="text-sm opacity-50">Settings and configuration</p>
    </div>

    <div class="card bg-base-200 p-4">
      <h3 class="font-semibold mb-2">Cursor Position (Multi-Window Test)</h3>
      <div class="flex flex-col gap-1">
        <span class="font-mono text-sm">
          Raw: ({$cursorPositionOnScreen.raw.x}, {$cursorPositionOnScreen.raw.y})
        </span>
        <span class="font-mono text-sm">
          Mapped: ({$cursorPositionOnScreen.mapped.x}, {$cursorPositionOnScreen.mapped.y})
        </span>
      </div>
    </div>
  </div>
</div>
