<script lang="ts">
  import { cursorPositionOnScreen } from "../../channels/cursor";
</script>

<div class="w-svw h-svh p-4 relative">
  <div
    class="size-max mx-auto bg-base-100 border-base-200 border px-4 py-3 rounded-xl"
  >
    <div class="flex flex-col text-center">
      <p class="text-xl">Friendolls</p>
      <p class="text-sm opacity-50">Scene Screen</p>
      <div class="mt-4">
        <span class="font-mono">
          Cursor: ({$cursorPositionOnScreen.raw.x}, {$cursorPositionOnScreen.raw
            .y})
        </span>
        <span class="font-mono">
          Cursor: ({$cursorPositionOnScreen.mapped.x}, {$cursorPositionOnScreen
            .mapped.y})
        </span>
      </div>
    </div>
  </div>
</div>
