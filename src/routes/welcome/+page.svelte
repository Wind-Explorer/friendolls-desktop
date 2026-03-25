<script lang="ts">
  import { commands, events } from "$lib/bindings";
  import { onDestroy, onMount } from "svelte";
  import DollPreview from "../app-menu/components/doll-preview.svelte";
  import ExternalLink from "../../assets/icons/external-link.svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import Google from "../../assets/icons/trademarks/google.svelte";
  import Discord from "../../assets/icons/trademarks/discord.svelte";

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

  const handleAuthFlowUpdated = ({
    payload,
  }: {
    payload: AuthFlowUpdatedPayload;
  }) => {
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

    errorMessage =
      payload.message ?? `Unable to sign in with ${providerLabel(provider)}.`;
  };

  onMount(async () => {
    unlistenAuthFlow = await events.authFlowUpdated.listen(
      handleAuthFlowUpdated,
    );
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
              <p class="text-xl opacity-70">meow? nyaaa!!</p>
            </div>
            <p class="text-2xl">a cute passive socialization layer!</p>
          </div>
          <div class="flex flex-col gap-3">
            <span>Sign in with</span>
            <div class="w-full flex flex-row gap-3">
              <button
                class="card relative hover:cursor-pointer items-center"
                onclick={() => startAuth("google")}
              >
                <div class="size-full btn btn-primary absolute z-0"></div>
                <div
                  class="flex flex-row justify-start items-center z-1 h-full p-1 pr-0"
                >
                  <div
                    class="bg-linear-to-t from-base-100/50 to-base-100 rounded-selector"
                  >
                    <div class="scale-70">
                      <Google />
                    </div>
                  </div>
                  <span class="text-xl px-3 text-primary-content">Google</span>
                </div>
              </button>
              <button
                class="card relative hover:cursor-pointer items-center"
                onclick={() => startAuth("discord")}
              >
                <div class="size-full btn btn-primary absolute z-0"></div>
                <div
                  class="flex flex-row justify-start items-center z-1 h-full p-1 pr-0"
                >
                  <div
                    class="bg-linear-to-t from-base-100/50 to-base-100 rounded-selector"
                  >
                    <div class="scale-70">
                      <Discord />
                    </div>
                  </div>
                  <span class="text-xl px-3 text-primary-content">Discord</span>
                </div>
              </button>
            </div>
          </div>

          {#if errorMessage}
            <p class="text-xs text-error max-w-48">{errorMessage}</p>
          {:else}
            <p class="text-xs opacity-70 max-w-48">
              An account is needed to connect you with friends.
            </p>
          {/if}

          <button
            class="btn btn-link p-0 btn-sm btn-primary w-max opacity-0 hover:opacity-100 transition-opacity"
            onclick={openClientConfig}
          >
            Client Configuration (Advanced)
          </button>
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
    <div class="flex flex-col scale-150 origin-bottom-right">
      <DollPreview dollColorScheme={{ body: "b7f2ff", outline: "496065" }} />
    </div>
  </div>
</div>
