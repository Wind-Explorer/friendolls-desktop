<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  type AuthConfig = {
    audience: string;
    auth_url: string;
  };

  type AppConfig = {
    api_base_url?: string | null;
    auth: AuthConfig;
  };

  let form: AppConfig = {
    api_base_url: "",
    auth: { audience: "", auth_url: "" },
  };

  let saving = false;
  let errorMessage = "";
  let successMessage = "";

  const loadConfig = async () => {
    try {
      const config = (await invoke("get_client_config")) as AppConfig;
      form = {
        api_base_url: config.api_base_url ?? "",
        auth: {
          audience: config.auth.audience,
          auth_url: config.auth.auth_url,
        },
      };
    } catch (err) {
      errorMessage = `Failed to load config: ${err}`;
    }
  };

  const save = async () => {
    if (saving) return;
    saving = true;
    errorMessage = "";
    successMessage = "";
    try {
      await invoke("save_client_config", {
        config: {
          api_base_url: form.api_base_url?.trim() || null,
          auth: {
            audience: form.auth.audience.trim(),
            auth_url: form.auth.auth_url.trim(),
          },
        },
      });

      successMessage = "Configuration saved. Please restart the app.";
      await invoke("restart_app");
    } catch (err) {
      errorMessage = `Failed to save config: ${err}`;
    } finally {
      saving = false;
    }
  };

  onMount(loadConfig);
</script>

<div class="p-6 flex flex-col gap-4">
  <div class="flex flex-col gap-1">
    <p class="text-xl font-semibold">Client Configuration</p>
    <p class="opacity-70 text-sm">Set custom API and auth endpoints.</p>
  </div>

  <div class="flex flex-col gap-3">
    <label class="flex flex-col gap-1">
      <span class="text-sm">API Base URL</span>
      <input
        class="input input-bordered"
        bind:value={form.api_base_url}
        placeholder="https://api.fdolls.adamcv.com"
      />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm">Auth URL</span>
      <input class="input input-bordered" bind:value={form.auth.auth_url} />
    </label>
    <label class="flex flex-col gap-1">
      <span class="text-sm">JWT Audience</span>
      <input class="input input-bordered" bind:value={form.auth.audience} />
    </label>
  </div>

  {#if errorMessage}
    <p class="text-sm text-error">{errorMessage}</p>
  {/if}
  {#if successMessage}
    <p class="text-sm text-success">{successMessage}</p>
  {/if}

  <div class="flex flex-row gap-2">
    <button
      class="btn"
      class:btn-disabled={saving}
      disabled={saving}
      on:click={save}
    >
      {saving ? "Saving..." : "Save"}
    </button>
  </div>
</div>
