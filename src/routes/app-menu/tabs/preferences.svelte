<script>
  import { invoke } from "@tauri-apps/api/core";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";

  let signingOut = false;
  let isChangingPassword = false;
  let passwordError = "";
  let passwordSuccess = "";
  let passwordForm = {
    currentPassword: "",
    newPassword: "",
    confirmPassword: "",
  };

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

  const openClientConfigManager = async () => {
    try {
      await invoke("open_client_config_manager");
    } catch (error) {
      console.error("Failed to open client config manager", error);
    }
  };

  const handleChangePassword = async () => {
    if (isChangingPassword) return;
    passwordError = "";
    passwordSuccess = "";

    if (!passwordForm.currentPassword || !passwordForm.newPassword) {
      passwordError = "Current and new password are required";
      return;
    }

    if (passwordForm.newPassword !== passwordForm.confirmPassword) {
      passwordError = "New password confirmation does not match";
      return;
    }

    isChangingPassword = true;
    try {
      await invoke("change_password", {
        currentPassword: passwordForm.currentPassword,
        newPassword: passwordForm.newPassword,
      });
      passwordSuccess = "Password updated";
      passwordForm.currentPassword = "";
      passwordForm.newPassword = "";
      passwordForm.confirmPassword = "";
    } catch (error) {
      console.error("Failed to change password", error);
      passwordError = error instanceof Error ? error.message : "Unable to update password";
    } finally {
      isChangingPassword = false;
    }
  };
</script>

<div class="size-full flex flex-col justify-between">
  <div class="flex flex-col gap-4">
    <p>{$appData?.user?.name}'s preferences</p>
    <div class="flex flex-row gap-2">
      <button class="btn" class:btn-disabled={signingOut} onclick={handleSignOut}>
        {signingOut ? "Signing out..." : "Sign out"}
      </button>
      <button class="btn btn-outline" onclick={openClientConfigManager}>
        Advanced options
      </button>
    </div>
    <div class="divider my-0"></div>
    <div class="flex flex-col gap-3 max-w-sm">
      <p class="text-sm opacity-70">Change password</p>
      <label class="flex flex-col gap-1">
        <span class="text-xs opacity-60">Current password</span>
        <input
          class="input input-bordered input-sm"
          type="password"
          autocomplete="current-password"
          bind:value={passwordForm.currentPassword}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs opacity-60">New password</span>
        <input
          class="input input-bordered input-sm"
          type="password"
          autocomplete="new-password"
          bind:value={passwordForm.newPassword}
        />
      </label>
      <label class="flex flex-col gap-1">
        <span class="text-xs opacity-60">Confirm new password</span>
        <input
          class="input input-bordered input-sm"
          type="password"
          autocomplete="new-password"
          bind:value={passwordForm.confirmPassword}
        />
      </label>
      <div class="flex flex-row gap-2 items-center">
        <button
          class="btn btn-sm"
          class:btn-disabled={isChangingPassword}
          disabled={isChangingPassword}
          onclick={handleChangePassword}
        >
          {isChangingPassword ? "Updating..." : "Update password"}
        </button>
        {#if passwordSuccess}
          <span class="text-xs text-success">{passwordSuccess}</span>
        {/if}
      </div>
      {#if passwordError}
        <p class="text-xs text-error">{passwordError}</p>
      {/if}
    </div>
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
