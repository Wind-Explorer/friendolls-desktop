<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import type { DollDto } from "../../../../types/bindings/DollDto";
  import type { UserProfile } from "../../../../types/bindings/UserProfile";
  import type { AppData } from "../../../../types/bindings/AppData";
  import DollsList from "./dolls-list.svelte";

  let dolls: DollDto[] = [];
  let user: UserProfile | null = null;
  let loading = false;
  let error: string | null = null;

  // We still keep the focus listener as a fallback, but the websocket events should handle most updates
  onMount(() => {
    refreshDolls();
    
    // Set up listeners
    const unlistenCreated = listen("doll.created", (event) => {
        console.log("Received doll.created event", event);
        refreshDolls();
    });

    const unlistenUpdated = listen("doll.updated", (event) => {
        console.log("Received doll.updated event", event);
        refreshDolls();
    });

    const unlistenDeleted = listen("doll.deleted", (event) => {
        console.log("Received doll.deleted event", event);
        refreshDolls();
    });

    // Listen for focus events to refresh data when returning from editor window
    window.addEventListener("focus", refreshDolls);
    
    return async () => {
      window.removeEventListener("focus", refreshDolls);
      (await unlistenCreated)();
      (await unlistenUpdated)();
      (await unlistenDeleted)();
    };
  });

  async function refreshDolls() {
    loading = true;
    try {
      dolls = await invoke("get_dolls");
      // Use refresh_app_data to ensure we get the latest user state (including activeDollId)
      // from the server, as the local state might be stale after updates.
      const appData: AppData = await invoke("refresh_app_data");
      user = appData.user;
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  async function openCreateModal() {
    await invoke("open_doll_editor_window", { dollId: null });
  }

  async function openEditModal(doll: DollDto) {
    await invoke("open_doll_editor_window", { dollId: doll.id });
  }

  async function handleSetActiveDoll(dollId: string) {
    try {
      await invoke("set_active_doll", { dollId });
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  async function handleRemoveActiveDoll() {
    try {
      await invoke("remove_active_doll");
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }
</script>

<div class="dolls-page flex flex-col gap-4">
  <div class="flex justify-between items-center">
    <h2 class="text-lg font-bold">Your Nekos</h2>
    <button class="btn btn-primary btn-sm" on:click={openCreateModal}>
      Add a Neko
    </button>
  </div>

  <DollsList
    {dolls}
    {user}
    {loading}
    {error}
    onEditDoll={openEditModal}
    onSetActiveDoll={handleSetActiveDoll}
    onRemoveActiveDoll={handleRemoveActiveDoll}
  />
</div>
