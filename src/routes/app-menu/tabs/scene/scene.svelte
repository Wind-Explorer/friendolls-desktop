<script lang="ts">
  import NekoView from "./neko-view.svelte";
  import NekoReposition from "./neko-reposition.svelte";
</script>

<div class="flex flex-col gap-4 w-full h-full">
  <p class="text-lg">Scene Configuration</p>
  <NekoView />
  <NekoReposition />
</div>
