<script lang="ts">
  import { cursorPositionOnScreen } from "../../events/cursor";
</script>

<div class="w-svw h-svh p-4 relative">
  <div
    class="size-max mx-auto bg-base-100 border-base-200 border px-4 py-3 rounded-xl"
  >
    <div class="flex flex-col text-center">
      <p class="text-xl">Friendolls</p>
      <p class="text-sm opacity-50">Scene Screen</p>
      <div class="mt-4 flex flex-col gap-1">
        <span class="font-mono text-sm">
          Raw: ({$cursorPositionOnScreen.raw.x}, {$cursorPositionOnScreen.raw
            .y})
        </span>
        <span class="font-mono text-sm">
          Mapped: ({$cursorPositionOnScreen.mapped.x}, {$cursorPositionOnScreen
            .mapped.y})
        </span>
      </div>
    </div>
  </div>
</div>
