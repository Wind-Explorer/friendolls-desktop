<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

  let isContinuing = false;

  const handleContinue = async () => {
    if (isContinuing) return;
    isContinuing = true;
    try {
      await invoke("start_auth_flow");
      await getCurrentWebviewWindow().close();
    } catch (error) {
      console.error("Failed to start auth flow", error);
      isContinuing = false;
    }
  };
</script>

<div class="w-full h-full bg-base-100 flex items-center justify-center p-8">
  <div class="card w-full max-w-md bg-base-200 shadow-lg">
    <div class="card-body gap-4">
      <div class="flex flex-col gap-1">
        <h1 class="text-2xl font-semibold">Friendolls</h1>
        <p class="text-base text-base-content/80">
          Passive social app connecting peers through mouse cursor interactions in the form of desktop pets.
        </p>
      </div>
      <div class="card-actions justify-end pt-2">
        <button class="btn btn-primary" onclick={handleContinue} disabled={isContinuing}>
          {#if isContinuing}
            Loading...
          {:else}
            Continue
          {/if}
        </button>
      </div>
    </div>
  </div>
</div>
