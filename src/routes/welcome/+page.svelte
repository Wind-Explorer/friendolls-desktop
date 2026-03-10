<script lang="ts">
  import { commands } from "$lib/bindings";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import DollPreview from "../app-menu/components/doll-preview.svelte";
  import ExternalLink from "../../assets/icons/external-link.svelte";

  let isContinuing = false;
  let useRegister = false;
  let errorMessage = "";
  let form = {
    email: "",
    password: "",
    name: "",
    username: "",
  };

  const normalizeError = (value: unknown) => {
    if (value instanceof Error) {
      return value.message;
    }
    return typeof value === "string" ? value : "Something went wrong";
  };

  const handleContinue = async () => {
    if (isContinuing) return;
    if (!form.email.trim() || !form.password) {
      errorMessage = "Email and password are required";
      return;
    }
    isContinuing = true;
    errorMessage = "";
    try {
      if (useRegister) {
        await commands.register(
          form.email.trim(),
          form.password,
          form.name.trim() || null,
          form.username.trim() || null,
        );
        useRegister = false;
        resetRegisterFields();
        form.password = "";
        return;
      }

      await commands.login(form.email.trim(), form.password);
      await getCurrentWebviewWindow().close();
    } catch (error) {
      console.error("Failed to authenticate", error);
      errorMessage = normalizeError(error);
    }
    isContinuing = false;
  };

  const resetRegisterFields = () => {
    form.name = "";
    form.username = "";
  };

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
          <div class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <label class="flex flex-col gap-1">
                <span class="text-xs opacity-60">Email</span>
                <input
                  class="input input-bordered input-sm"
                  type="email"
                  autocomplete="email"
                  bind:value={form.email}
                  placeholder="you@example.com"
                />
              </label>
              <label class="flex flex-col gap-1">
                <span class="text-xs opacity-60">Password</span>
                <input
                  class="input input-bordered input-sm"
                  type="password"
                  autocomplete={useRegister
                    ? "new-password"
                    : "current-password"}
                  bind:value={form.password}
                  placeholder="••••••••"
                />
              </label>
              {#if useRegister}
                <label class="flex flex-col gap-1">
                  <span class="text-xs opacity-60">Name (optional)</span>
                  <input
                    class="input input-bordered input-sm"
                    autocomplete="name"
                    bind:value={form.name}
                  />
                </label>
                <label class="flex flex-col gap-1">
                  <span class="text-xs opacity-60">Username (optional)</span>
                  <input
                    class="input input-bordered input-sm"
                    autocomplete="username"
                    bind:value={form.username}
                  />
                </label>
              {/if}
            </div>
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
                {useRegister ? "Create account" : "Sign in"}
              {/if}
            </button>
            <button
              class="btn btn-ghost btn-sm px-0 justify-start"
              onclick={() => {
                useRegister = !useRegister;
                errorMessage = "";
                if (!useRegister) {
                  resetRegisterFields();
                }
              }}
            >
              {useRegister
                ? "Already have an account? Sign in"
                : "New here? Create an account"}
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
            <p class="text-xs opacity-50 max-w-60">
              An account is needed to identify you for connecting with friends.
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
