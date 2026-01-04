<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  let errorMessage = "";
  let unlisten: (() => void) | null = null;
  let isRestarting = false;

  onMount(async () => {
    unlisten = await listen<string>("health-error", (event) => {
      errorMessage = event.payload;
    });
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });

  const tryAgain = async () => {
    if (isRestarting) return;
    isRestarting = true;
    errorMessage = "";
    try {
      await invoke("restart_app");
    } catch (err) {
      errorMessage = `Restart failed: ${err}`;
      isRestarting = false;
    }
  };
</script>

<div class="size-full p-4">
  <div class="flex flex-col gap-4 size-full justify-between">
  <div class="flex flex-col gap-2">
    <p class="text-md font-light">Something is not right...</p>
    <p class="opacity-70 text-3xl font-bold">
      Seems like the server is inaccessible. Check your network?
    </p>
    {#if errorMessage}
      <p class="text-sm opacity-70 wrap-break-word">{errorMessage}</p>
    {/if}
  </div>
  <div class="flex flex-row gap-2">
    <button
      class="btn"
      class:btn-disabled={isRestarting}
      disabled={isRestarting}
      onclick={tryAgain}
    >
      {#if isRestarting}
        Retrying…
      {:else}
        Try again
      {/if}
    </button>
    <button
      class="btn btn-outline"
      onclick={async () => {
        try {
          await invoke("open_client_config_manager");
        } catch (err) {
          errorMessage = `Failed to open config manager: ${err}`;
        }
      }}
    >
      Advanced options
    </button>
  </div>

  </div>
</div>
