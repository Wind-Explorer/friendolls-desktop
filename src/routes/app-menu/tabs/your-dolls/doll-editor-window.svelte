<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import type { DollDto } from "../../../../types/bindings/DollDto";
  import type { CreateDollDto } from "../../../../types/bindings/CreateDollDto";
  import type { UpdateDollDto } from "../../../../types/bindings/UpdateDollDto";
  import DollPreview from "../DollPreview.svelte";

  let mode: "create" | "edit" = "create";
  let dollId: string | null = null;
  let loading = true;
  let error: string | null = null;

  let name = "";
  let bodyColor = "#FFFFFF";
  let outlineColor = "#000000";

  onMount(async () => {
    // Check URL search params for ID
    const urlParams = new URLSearchParams(window.location.search);
    const id = urlParams.get("id");

    if (id) {
      mode = "edit";
      dollId = id;
      await fetchDoll(id);
    } else {
      mode = "create";
      loading = false;
    }
  });

  async function fetchDoll(id: string) {
    loading = true;
    try {
      const doll: DollDto = await invoke("get_doll", { id });
      name = doll.name;
      bodyColor = doll.configuration.colorScheme.body;
      outlineColor = doll.configuration.colorScheme.outline;
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error("Failed to fetch doll:", e);
    } finally {
      loading = false;
    }
  }

  async function handleSave() {
    if (!name.trim()) return;

    try {
      if (mode === "create") {
        const dto: CreateDollDto = {
          name,
          configuration: {
            colorScheme: {
              body: bodyColor,
              outline: outlineColor,
            },
          },
        };
        await invoke("create_doll", { dto });
      } else if (dollId) {
        const dto: UpdateDollDto = {
          name,
          configuration: {
            colorScheme: {
              body: bodyColor,
              outline: outlineColor,
            },
          },
        };
        await invoke("update_doll", { id: dollId, dto });
      }

      // Close window on success
      await getCurrentWebviewWindow().close();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error("Failed to save doll:", e);
    }
  }

  async function handleCancel() {
    await getCurrentWebviewWindow().close();
  }
</script>

<div class="w-screen h-screen bg-base-100 flex flex-col">
  {#if loading}
    <div class="flex h-full items-center justify-center">
      <span class="loading loading-spinner loading-lg"></span>
    </div>
  {:else if error}
    <div class="flex h-full flex-col items-center justify-center p-4">
      <div class="alert alert-error">
        <span>{error}</span>
      </div>
      <button class="btn btn-sm mt-4" on:click={handleCancel}>Close</button>
    </div>
  {:else}
    <div class="h-full w-full p-4 flex flex-col">
      <div class="form-control w-full">
        <label class="label">
          <span class="label-text">Name</span>
        </label>
        <input
          type="text"
          placeholder="Doll Name"
          class="input input-bordered w-full"
          bind:value={name}
        />
      </div>
      <div class="flex justify-center mt-4">
        <DollPreview {bodyColor} {outlineColor} />
      </div>
      <div class="form-control w-full mt-2">
        <label class="label">
          <span class="label-text">Body Color</span>
        </label>
        <div class="flex gap-2">
          <input
            type="color"
            class="input input-bordered w-12 p-1 h-10"
            bind:value={bodyColor}
          />
          <input
            type="text"
            class="input input-bordered w-full"
            bind:value={bodyColor}
          />
        </div>
      </div>
      <div class="form-control w-full mt-2">
        <label class="label">
          <span class="label-text">Outline Color</span>
        </label>
        <div class="flex gap-2">
          <input
            type="color"
            class="input input-bordered w-12 p-1 h-10"
            bind:value={outlineColor}
          />
          <input
            type="text"
            class="input input-bordered w-full"
            bind:value={outlineColor}
          />
        </div>
      </div>
      <div class="mt-auto pt-4 flex justify-end gap-2">
        <button class="btn" on:click={handleCancel}>Cancel</button>
        <button
          class="btn btn-primary"
          on:click={handleSave}
          disabled={!name.trim()}
        >
          {#if mode === "create"}
            Create
          {:else}
            Save
          {/if}
        </button>
      </div>
    </div>
  {/if}
</div>