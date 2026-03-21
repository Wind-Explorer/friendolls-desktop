<script lang="ts">
  import { onMount } from "svelte";
  import {
    commands,
    type AppConfig,
    type KeyboardAccelerator,
  } from "$lib/bindings";
  import {
    MODIFIER_CODES,
    SCENE_INTERACTIVITY_ACTION,
    acceleratorsEqual,
    formatAcceleratorLabel,
    getAcceleratorForAction,
    getModifiersFromEvent,
    keyFromKeyboardCode,
    normalizeAccelerator,
  } from "$lib/utils/accelerators";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";

  let signingOut = false;
  let appConfig: AppConfig | null = null;
  let accelerator: KeyboardAccelerator | null = null;
  let captureMode = false;
  let capturePreviewLabel = "";
  let acceleratorInput: HTMLInputElement | null = null;
  let acceleratorLabel = "";
  let acceleratorError = "";
  let acceleratorSuccess = "";
  let acceleratorDirty = false;
  let acceleratorSaving = false;

  const loadConfig = async () => {
    try {
      const config = await commands.getClientConfig();
      appConfig = config;
      accelerator = getAcceleratorForAction(config, SCENE_INTERACTIVITY_ACTION);
    } catch (error) {
      console.error("Failed to load client config", error);
      acceleratorError = "Failed to load current accelerator settings.";
    }
  };

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

  const beginCapture = () => {
    captureMode = true;
    capturePreviewLabel = "";
    acceleratorError = "";
    acceleratorSuccess = "";
    setTimeout(() => acceleratorInput?.focus(), 0);
  };

  const stopCapture = () => {
    captureMode = false;
    capturePreviewLabel = "";
    saveAccelerator();
  };

  const saveAccelerator = async () => {
    if (!appConfig || !accelerator || acceleratorSaving) return;
    acceleratorSaving = true;
    acceleratorError = "";
    acceleratorSuccess = "";

    try {
      const nextConfig: AppConfig = {
        ...appConfig,
        accelerators: {
          ...(appConfig.accelerators ?? {}),
          [SCENE_INTERACTIVITY_ACTION]: accelerator,
        },
      };
      await commands.saveClientConfig(nextConfig);
      appConfig = nextConfig;
      acceleratorSuccess = "Accelerator saved.";
    } catch (error) {
      console.error("Failed to save accelerator", error);
      acceleratorError = "Failed to save accelerator.";
    } finally {
      acceleratorSaving = false;
    }
  };

  const onAcceleratorCapture = async (event: KeyboardEvent) => {
    if (!captureMode || acceleratorSaving) return;

    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      stopCapture();
      return;
    }

    const modifiers = getModifiersFromEvent(event);
    const key = keyFromKeyboardCode(event.code);
    const modifierOnlyPress = MODIFIER_CODES.has(event.code);

    if (modifiers.length === 0) {
      capturePreviewLabel = "";
      acceleratorError = "Accelerator must include at least one modifier key.";
      return;
    }

    if (modifierOnlyPress) {
      const nextAccelerator = normalizeAccelerator({
        modifiers,
        key: null,
      });
      accelerator = nextAccelerator;
      capturePreviewLabel = formatAcceleratorLabel(nextAccelerator);
      acceleratorSuccess = "";
      return;
    }

    if (!key) {
      acceleratorError = "That key is not supported for accelerator combos yet.";
      return;
    }

    const nextAccelerator = normalizeAccelerator({ modifiers, key });
    accelerator = nextAccelerator;
    capturePreviewLabel = formatAcceleratorLabel(nextAccelerator);
    acceleratorSuccess = "";
  };

  $: acceleratorLabel = accelerator ? formatAcceleratorLabel(accelerator) : "";
  $: acceleratorDirty = appConfig
    ? accelerator && appConfig.accelerators?.[SCENE_INTERACTIVITY_ACTION]
      ? !acceleratorsEqual(
        accelerator,
        appConfig.accelerators?.[SCENE_INTERACTIVITY_ACTION],
      )
      : false
    : false;

  onMount(() => {
    loadConfig();

    const handleKeydown = (event: KeyboardEvent) => {
      void onAcceleratorCapture(event);
    };

    window.addEventListener("keydown", handleKeydown, true);

    return () => {
      window.removeEventListener("keydown", handleKeydown, true);
    };
  });
</script>

<div class="size-full flex flex-col justify-between">
  <div class="flex flex-col gap-4 max-w-md">
    <p>{$appData?.user?.name}'s preferences</p>
    <label class="flex flex-col gap-2">
      <span class="text-sm">Scene Interactivity Accelerator</span>
      <div class="flex flex-row items-center gap-2">
        <input
          class="input input-bordered flex-1"
          readonly
          bind:this={acceleratorInput}
          value={captureMode
            ? capturePreviewLabel || "Press your shortcut..."
            : acceleratorLabel}
        />
        {#if captureMode}
          <button
            class="btn btn-outline"
            type="button"
            disabled={acceleratorSaving}
            onclick={stopCapture}
          >
            {acceleratorSaving ? "Saving..." : "Stop Record"}
          </button>
        {:else}
          <div class="flex flex-row gap-2">
            <button
              class="btn btn-outline"
              type="button"
              disabled={!appConfig || acceleratorSaving}
              onclick={beginCapture}
            >
              Record
            </button>
          </div>
        {/if}
      </div>
      <span class="text-xs opacity-70">
        Requires at least one modifier (Cmd, Alt, Ctrl, Shift). Press Escape to
        cancel recording.
      </span>
      {#if acceleratorError}
        <span class="text-xs text-error">{acceleratorError}</span>
      {/if}
      {#if acceleratorSuccess}
        <span class="text-xs text-success">{acceleratorSuccess}</span>
      {/if}
    </label>
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
