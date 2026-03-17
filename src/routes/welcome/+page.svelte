<script lang="ts">
  import { commands, events } from "$lib/bindings";
  import { onDestroy, onMount } from "svelte";
  import DollPreview from "../app-menu/components/doll-preview.svelte";
  import ExternalLink from "../../assets/icons/external-link.svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";

  let activeProvider: "google" | "discord" | null = null;
  let errorMessage = "";
  let unlistenAuthFlow: UnlistenFn | null = null;

  type AuthFlowUpdatedPayload = {
    provider: string;
    status: "started" | "succeeded" | "failed" | "cancelled";
    message: string | null;
  };

  const normalizeError = (value: unknown) => {
    if (value instanceof Error) {
      return value.message;
    }

    return typeof value === "string" ? value : "Something went wrong";
  };

  const startAuth = async (provider: "google" | "discord") => {
    activeProvider = provider;
    errorMessage = "";

    try {
      if (provider === "google") {
        await commands.startGoogleAuth();
      } else {
        await commands.startDiscordAuth();
      }
    } catch (error) {
      console.error(`Failed to start ${provider} auth`, error);
      errorMessage = normalizeError(error);
      if (activeProvider === provider) {
        activeProvider = null;
      }
    }
  };

  const providerLabel = (provider: "google" | "discord") =>
    provider === "google" ? "Google" : "Discord";

  const handleAuthFlowUpdated = ({ payload }: { payload: AuthFlowUpdatedPayload }) => {
    const provider = payload.provider as "google" | "discord";
    if (activeProvider !== provider) {
      return;
    }

    if (payload.status === "started") {
      return;
    }

    activeProvider = null;

    if (payload.status === "succeeded") {
      errorMessage = "";
      return;
    }

    errorMessage = payload.message ?? `Unable to sign in with ${providerLabel(provider)}.`;
  };

  onMount(async () => {
    unlistenAuthFlow = await events.authFlowUpdated.listen(handleAuthFlowUpdated);
  });

  onDestroy(() => {
    unlistenAuthFlow?.();
  });

  const openClientConfig = async () => {
    try {
      await commands.openClientConfig();
    } catch (error) {
      console.error("Failed to open client config", error);
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
          <div class="flex flex-col gap-3 max-w-80">
            <button
              class="btn btn-primary btn-xl justify-between"
              onclick={() => startAuth("google")}
            >
              <span>{activeProvider === "google" ? "Restart Google sign-in" : "Continue with Google"}</span>
              <div class="scale-70">
                <ExternalLink />
              </div>
            </button>
            <button
              class="btn btn-outline btn-xl justify-between"
              onclick={() => startAuth("discord")}
            >
              <span>{activeProvider === "discord" ? "Restart Discord sign-in" : "Continue with Discord"}</span>
              <div class="scale-70">
                <ExternalLink />
              </div>
            </button>
            <button
              class="btn btn-link p-0 btn-sm text-base-content w-max"
              onclick={openClientConfig}
            >
              Advanced options
            </button>
          </div>

          {#if errorMessage}
            <p class="text-xs text-error max-w-72">{errorMessage}</p>
          {:else}
            <p class="text-xs opacity-50 max-w-72">
              {#if activeProvider}
                Friendolls is waiting for the browser callback. Click either button again to restart sign-in at any time.
              {:else}
                Sign in through your browser, then return here once Friendolls finishes the handshake.
              {/if}
            </p>
          {/if}
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
      <DollPreview dollColorScheme={{ body: "b7f2ff", outline: "496065" }} />
    </div>
  </div>
</div>
