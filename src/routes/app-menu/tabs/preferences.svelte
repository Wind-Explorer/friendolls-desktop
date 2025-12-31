<script>
  import { invoke } from "@tauri-apps/api/core";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";

  let signingOut = false;

  async function handleSignOut() {
    if (signingOut) return;
    signingOut = true;
    try {
      await invoke("logout_and_restart");
    } catch (error) {
      console.error("Failed to sign out", error);
      signingOut = false;
    }
  }
</script>

<div class="size-full flex flex-col justify-between">
  <div class="flex flex-col gap-2">
    <p>{$appData?.user?.name}'s preferences</p>
    <button class="btn" class:btn-disabled={signingOut} onclick={handleSignOut}>
      {signingOut ? "Signing out..." : "Sign out"}
    </button>
  </div>
  <div class="w-full flex flex-row justify-between">
    <div></div>
    <div>
      <button
        class="btn btn-error btn-square btn-soft"
        onclick={async () => {
          await invoke("quit_app");
        }}
      >
        <div class="scale-50">
          <Power />
        </div>
      </button>
    </div>
  </div>
</div>
