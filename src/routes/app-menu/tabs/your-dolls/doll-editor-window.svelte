<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import DollEditor from "./doll-editor.svelte";
  import type { DollDto } from "../../../../types/bindings/DollDto";
  import type { CreateDollDto } from "../../../../types/bindings/CreateDollDto";
  import type { UpdateDollDto } from "../../../../types/bindings/UpdateDollDto";

  let mode: "create" | "edit" = "create";
  let dollId: string | null = null;
  let loading = true;
  let error: string | null = null;

  let initialName = "";
  let initialBodyColor = "#FFFFFF";
  let initialOutlineColor = "#000000";

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
      initialName = doll.name;
      initialBodyColor = doll.configuration.colorScheme.body;
      initialOutlineColor = doll.configuration.colorScheme.outline;
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
      console.error("Failed to fetch doll:", e);
    } finally {
      loading = false;
    }
  }

  async function handleSave(
    name: string,
    bodyColor: string,
    outlineColor: string,
  ) {
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

<div class="w-screen h-screen bg-base-100">
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
    <DollEditor
      isOpen={true}
      standalone={true}
      {mode}
      {initialName}
      {initialBodyColor}
      {initialOutlineColor}
      onSave={handleSave}
      onCancel={handleCancel}
    />
  {/if}
</div>
