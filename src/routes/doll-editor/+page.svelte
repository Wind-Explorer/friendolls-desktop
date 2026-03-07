<script lang="ts">
  import { onMount } from "svelte";
  import {
    commands,
    type CreateDollDto,
    type DollDto,
    type UpdateDollDto,
  } from "$lib/bindings";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import DollPreview from "../app-menu/components/doll-preview.svelte";

  let mode: "create" | "edit" = "create";
  let dollId: string | null = null;
  let loading = true;
  let error: string | null = null;
  let saving = false;

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
      const doll: DollDto = await commands.getDoll(id);
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

    saving = true;
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
        await commands.createDoll(dto);
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
        await commands.updateDoll(dollId, dto);
      }

      // Close window on success
      await getCurrentWebviewWindow().close();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error("Failed to save doll:", e);
    } finally {
      saving = false;
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
    <div class="h-full w-full p-4 gap-4 flex flex-col">
      <div class="flex justify-center mt-4">
        <DollPreview {bodyColor} {outlineColor} />
      </div>
      <div class="form-control w-full">
        <label class="label" for="name-input">
          <span class="label-text">Name</span>
        </label>
        <input
          id="name-input"
          type="text"
          placeholder="Doll Name"
          class="input input-bordered w-full"
          bind:value={name}
          disabled={saving}
        />
      </div>
      <div class="form-control w-full">
        <label class="label" for="body-color-input">
          <span class="label-text">Body Color</span>
        </label>
        <div class="flex gap-2">
          <input
            id="body-color-input"
            type="color"
            class="input input-bordered w-10 p-0"
            bind:value={bodyColor}
            disabled={saving}
          />
          <input
            type="text"
            class="input input-bordered w-full"
            bind:value={bodyColor}
            disabled={saving}
          />
        </div>
      </div>
      <div class="form-control w-full">
        <label class="label" for="outline-color-input">
          <span class="label-text">Outline Color</span>
        </label>
        <div class="flex gap-2">
          <input
            id="outline-color-input"
            type="color"
            class="input input-bordered w-10 p-0"
            bind:value={outlineColor}
            disabled={saving}
          />
          <input
            type="text"
            class="input input-bordered w-full"
            bind:value={outlineColor}
            disabled={saving}
          />
        </div>
      </div>
      <div class="mt-auto flex justify-end gap-2">
        <button class="btn" on:click={handleCancel} disabled={saving}
          >Cancel</button
        >
        <button
          class="btn btn-primary"
          on:click={handleSave}
          disabled={!name.trim() || saving}
        >
          {#if saving}
            <span class="loading loading-spinner loading-sm"></span> Saving...
          {:else if mode === "create"}
            Create
          {:else}
            Save
          {/if}
        </button>
      </div>
    </div>
  {/if}
</div>
