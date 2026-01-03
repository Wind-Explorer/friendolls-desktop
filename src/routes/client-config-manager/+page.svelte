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
  let restartError = "";

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

  const validate = () => {
    if (!form.auth.auth_url.trim()) {
      return "Auth URL is required";
    }

    try {
      const parsed = new URL(form.auth.auth_url.trim());
      if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
        return "Auth URL must start with http or https";
      }
    } catch (e) {
      return "Auth URL must be a valid URL";
    }

    if (!form.auth.audience.trim()) {
      return "JWT audience is required";
    }

    if (form.api_base_url?.trim()) {
      try {
        const parsed = new URL(
          form.api_base_url.trim().startsWith("http")
            ? form.api_base_url.trim()
            : `https://${form.api_base_url.trim()}`
        );
        if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
          return "API base URL must start with http or https";
        }
      } catch (e) {
        return "API base URL must be a valid URL";
      }
    }

    return "";
  };

  const save = async () => {
    if (saving) return;
    errorMessage = validate();
    if (errorMessage) return;

    saving = true;
    errorMessage = "";
    successMessage = "";
    restartError = "";
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

      successMessage = "Configuration saved. Restart to apply changes.";
    } catch (err) {
      errorMessage = `Failed to save config: ${err}`;
    } finally {
      saving = false;
    }
  };

  const restart = async () => {
    restartError = "";
    try {
      await invoke("restart_app");
    } catch (err) {
      restartError = `Restart failed: ${err}`;
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
  {#if restartError}
    <p class="text-sm text-error">{restartError}</p>
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
    <button class="btn btn-outline" on:click={restart}>
      Restart app
    </button>
  </div>
</div>

