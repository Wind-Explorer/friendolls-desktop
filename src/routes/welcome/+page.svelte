<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import DollPreview from "../app-menu/components/doll-preview.svelte";
  import ExternalLink from "../../assets/icons/external-link.svelte";

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

  const openClientConfigManager = async () => {
    try {
      await invoke("open_client_config_manager");
    } catch (error) {
      console.error("Failed to open client config manager", error);
    }
  };
</script>

<div
  class="size-full max-w-full max-h-full overflow-hidden relative bg-linear-to-br from-base-100 to-[#b7f2ff77]"
>
  <div class="flex flex-row gap-2 justify-between size-full p-6">
    <div class="flex flex-col justify-between">
      <div class="flex flex-col gap-2 h-full">
        <div class="flex flex-col gap-6">
          <div class="flex flex-col gap-2">
            <div class="flex flex-row gap-2">
              <p class="text-xl font-light">meow? nyaaa!!</p>
            </div>
            <p class="opacity-70 text-3xl font-bold">
              a cute passive socialization layer!
            </p>
          </div>
          <div class="flex flex-col gap-4 *:w-max">
            <button
              class="btn btn-primary btn-xl"
              onclick={handleContinue}
              disabled={isContinuing}
            >
              {#if isContinuing}
                Loading...
              {:else}
                <div class="scale-70">
                  <ExternalLink />
                </div>
                Sign in
              {/if}
            </button>
            <button
              class="btn btn-link p-0 btn-sm text-base-content"
              onclick={openClientConfigManager}
            >
              Advanced options
            </button>
          </div>

          <p class="text-xs opacity-50 max-w-60">
            An account is needed to identify you for connecting with friends.
          </p>
        </div>
      </div>
      <div>
        <p class="uppercase">Pre-release</p>
      </div>
    </div>
  </div>
  <div
    class="absolute pointer-events-none bottom-6 right-6 flex flex-col gap-1 justify-between"
  >
    <div></div>
    <div class="flex flex-col scale-200 origin-bottom-right">
      <DollPreview bodyColor="b7f2ff" outlineColor="496065" />
    </div>
  </div>
</div>
