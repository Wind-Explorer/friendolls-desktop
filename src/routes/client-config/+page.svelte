<script lang="ts">
  import { onMount } from "svelte";
  import {
    commands,
    type AppConfig,
    type SceneInteractivityHotkey,
  } from "$lib/bindings";

  const DEFAULT_HOTKEY: SceneInteractivityHotkey = {
    modifiers: ["alt"],
    key: null,
  };

  const normalizeHotkey = (
    hotkey: SceneInteractivityHotkey | null | undefined,
  ): SceneInteractivityHotkey => {
    if (!hotkey) return { ...DEFAULT_HOTKEY };

    const uniqueModifiers = [...new Set(hotkey.modifiers ?? [])].sort();
    if (uniqueModifiers.length === 0) {
      return { ...DEFAULT_HOTKEY };
    }

    return {
      modifiers: uniqueModifiers,
      key: hotkey.key ?? null,
    };
  };

  let form: AppConfig = {
    api_base_url: "",
    debug_mode: false,
    scene_interactivity_hotkey: { ...DEFAULT_HOTKEY },
  };

  let saving = false;
  let errorMessage = "";
  let successMessage = "";
  let restartError = "";

  const loadConfig = async () => {
    try {
      const config = await commands.getClientConfig();
      form = {
        api_base_url: config.api_base_url ?? "",
        debug_mode: config.debug_mode ?? false,
        scene_interactivity_hotkey: normalizeHotkey(
          config.scene_interactivity_hotkey,
        ),
      };
    } catch (err) {
      errorMessage = `Failed to load config: ${err}`;
    }
  };

  const validate = () => {
    if (form.api_base_url?.trim()) {
      try {
        const parsed = new URL(
          form.api_base_url.trim().startsWith("http")
            ? form.api_base_url.trim()
            : `https://${form.api_base_url.trim()}`,
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
      await commands.saveClientConfig({
        api_base_url: form.api_base_url?.trim() || null,
        debug_mode: form.debug_mode,
        scene_interactivity_hotkey: normalizeHotkey(
          form.scene_interactivity_hotkey,
        ),
      });

      successMessage = "Success. Restart to apply changes.";
    } catch (err) {
      errorMessage = `Failed to save config: ${err}`;
    } finally {
      saving = false;
    }
  };

  const restart = async () => {
    restartError = "";
    try {
      await commands.restartApp();
    } catch (err) {
      restartError = `Restart failed: ${err}`;
    }
  };

  onMount(loadConfig);
</script>

<div class="p-6 flex flex-col gap-4 w-full h-full justify-between">
  <div class="flex flex-col gap-4 w-full">
    <div class="flex flex-col gap-1">
      <p class="text-xl font-semibold">Client Configuration</p>
      <p class="opacity-70 text-sm">Set a custom API endpoint.</p>
    </div>

    <div class="flex flex-col gap-3">
      <label class="flex flex-col gap-1">
        <span class="text-sm">API Base URL</span>
        <input
          class="input input-bordered"
          bind:value={form.api_base_url}
          placeholder="https://api.friendolls.adamcv.com"
        />
      </label>

      <label class="flex flex-row items-center gap-3 cursor-pointer">
        <span class="text-sm">Debug Mode</span>
        <input
          type="checkbox"
          class="checkbox checkbox-primary"
          bind:checked={form.debug_mode}
        />
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
  </div>

  <div class="flex flex-row gap-2 w-full justify-end">
    <button class="btn btn-outline" on:click={restart}> Restart app </button>
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
