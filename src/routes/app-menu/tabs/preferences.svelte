<script lang="ts">
  import { commands } from "$lib/bindings";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";

  let signingOut = false;

  async function handleSignOut() {
    if (signingOut) return;
    signingOut = true;
    try {
      await commands.logoutAndRestart();
    } catch (error) {
      console.error("Failed to sign out", error);
      signingOut = false;
    }
  }

  const openClientConfig = async () => {
    try {
      await commands.openClientConfig();
    } catch (error) {
      console.error("Failed to open client config", error);
    }
  };
</script>

<div class="size-full flex flex-col justify-between">
  <div class="flex flex-col gap-4 max-w-md">
    <p>{$appData?.user?.name}'s preferences</p>
    <div class="flex flex-row gap-2">
      <button
        class="btn"
        class:btn-disabled={signingOut}
        onclick={handleSignOut}
      >
        {signingOut ? "Signing out..." : "Sign out"}
      </button>
      <button class="btn btn-outline" onclick={openClientConfig}>
        Advanced options
      </button>
    </div>
  </div>
  <div class="w-full flex flex-row justify-between">
    <div></div>
    <div>
      <button
        class="btn btn-error btn-square btn-soft"
        onclick={async () => {
          await commands.quitApp();
        }}
      >
        <div class="scale-50">
          <Power />
        </div>
      </button>
    </div>
  </div>
</div>
