<script>
    import { browser } from '$app/environment';
    import { onMount, onDestroy } from 'svelte';
    import { initCursorTracking, stopCursorTracking } from '../events/cursor';

    let { children } = $props();
    if (browser) {
      onMount(async () => {
        try {
          await initCursorTracking();
        } catch (err) {
          console.error("[Scene] Failed to initialize cursor tracking:", err);
        }
      });

      onDestroy(() => {
        stopCursorTracking();
      });
    }

</script>

<div class="size-full bg-transparent">
    {@render children?.()}
</div>
