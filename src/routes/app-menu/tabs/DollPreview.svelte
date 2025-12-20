<script lang="ts">
  import { onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { fade } from "svelte/transition";

  export let bodyColor: string;
  export let outlineColor: string;
  export let applyTexture: boolean = true;

  let previewBase64: string | null = null;
  let error: string | null = null;
  let debounceTimer: number | null = null;

  function generatePreview() {
    error = null;
    try {
      invoke<string>("recolor_gif_base64", {
        whiteColorHex: bodyColor,
        blackColorHex: outlineColor,
        applyTexture,
      })
        .then((result) => {
          previewBase64 = result;
        })
        .catch((e) => {
          error = (e as Error)?.message ?? String(e);
          console.error(e);
        });
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error(e);
    }
  }

  function debouncedGeneratePreview() {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      generatePreview();
    }, 300); // Adjust debounce delay as needed (300ms is a common starting point)
  }

  $: if (bodyColor && outlineColor) {
    debouncedGeneratePreview();
  }

  onDestroy(() => {
    if (debounceTimer) clearTimeout(debounceTimer);
  });
</script>

<div class="size-32">
  {#if error}
    <div class="size-full flex justify-center items-center text-xs text-error">
      Error
    </div>
  {:else if previewBase64}
    <div
      class="size-full bg-cover bg-center pixelated"
      style="background-image: url('data:image/gif;base64,{previewBase64}')"
    ></div>
  {:else}
    <div class="size-full skeleton" style:background-color={bodyColor}></div>
  {/if}
</div>
