<script>
  import { invoke } from "@tauri-apps/api/core";
</script>

<div class="size-full p-4">
  <div class="flex flex-col gap-4 size-full justify-between">
    <div class="flex flex-col gap-2">
      <p class="text-md font-light">Something is not right...</p>
      <p class="opacity-70 text-3xl font-bold">
        Seems like the server is inaccessible. Check your network?
      </p>
    </div>
    <button
      class="btn"
      onclick={() => {
        console.log("Retrying server health detection");
        invoke("restart_app");
      }}>Try again</button
    >
  </div>
</div>
