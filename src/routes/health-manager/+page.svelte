<script lang="ts">
  import { onMount } from "svelte";
  import { commands } from "$lib/bindings";
  import { page } from "$app/stores";

  let errorMessage = "";
  let isRetrying = false;

  onMount(() => {
    errorMessage = $page.url.searchParams.get("err") || "";
  });

  const tryAgain = async () => {
    if (isRetrying) return;
    isRetrying = true;
    errorMessage = "";
    try {
      await commands.retryConnection();
    } catch (err) {
      errorMessage = `${err}`;
      isRetrying = false;
    }
  };
</script>

<div class="size-full p-4">
  <div class="flex flex-col gap-4 size-full justify-between">
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-2">
        <p class="text-md">Something is not right...</p>
        <p class="opacity-70 text-3xl">
          Seems like the server is inaccessible. Check your network?
        </p>
      </div>
      {#if errorMessage}
        <p class="text-xs opacity-70 wrap-break-word">
          {errorMessage}
        </p>
      {/if}
    </div>
    <div class="flex flex-row gap-2">
      <button
        class="btn"
        class:btn-disabled={isRetrying}
        disabled={isRetrying}
        onclick={tryAgain}
      >
        {#if isRetrying}
          Retrying…
        {:else}
          Try again
        {/if}
      </button>
      <button
        class="btn btn-outline"
        onclick={async () => {
          try {
            await commands.openClientConfig();
          } catch (err) {
            errorMessage = `Failed to open config: ${err}`;
          }
        }}
      >
        Advanced options
      </button>
    </div>
  </div>
</div>
