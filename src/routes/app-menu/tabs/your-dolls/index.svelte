<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { DollDto } from "../../../../types/bindings/DollDto";
  import type { UserProfile } from "../../../../types/bindings/UserProfile";
  import type { AppData } from "../../../../types/bindings/AppData";
  import type { CreateDollDto } from "../../../../types/bindings/CreateDollDto";
  import type { UpdateDollDto } from "../../../../types/bindings/UpdateDollDto";
  import DollsList from "./dolls-list.svelte";
  import DollEditor from "./doll-editor.svelte";

  let dolls: DollDto[] = [];
  let user: UserProfile | null = null;
  let loading = false;
  let error: string | null = null;
  let isEditorOpen = false;
  let editorMode: "create" | "edit" = "create";
  let editingDollId: string | null = null;
  let editorInitialName = "";
  let editorInitialBodyColor = "#FFFFFF";
  let editorInitialOutlineColor = "#000000";

  onMount(() => {
    refreshDolls();
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

  function openCreateModal() {
    editorMode = "create";
    editorInitialName = "";
    editorInitialBodyColor = "#FFFFFF";
    editorInitialOutlineColor = "#000000";
    isEditorOpen = true;
  }

  function openEditModal(doll: DollDto) {
    editorMode = "edit";
    editingDollId = doll.id;
    editorInitialName = doll.name;
    editorInitialBodyColor = doll.configuration.colorScheme.body;
    editorInitialOutlineColor = doll.configuration.colorScheme.outline;
    isEditorOpen = true;
  }

  function closeEditor() {
    isEditorOpen = false;
    editingDollId = null;
  }

  async function handleSave(
    name: string,
    bodyColor: string,
    outlineColor: string,
  ) {
    if (editorMode === "create") {
      await handleCreateDoll(name, bodyColor, outlineColor);
    } else {
      await handleUpdateDoll(name, bodyColor, outlineColor);
    }
  }

  async function handleCreateDoll(
    name: string,
    bodyColor: string,
    outlineColor: string,
  ) {
    try {
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
      closeEditor();
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  async function handleUpdateDoll(
    name: string,
    bodyColor: string,
    outlineColor: string,
  ) {
    if (!editingDollId) return;

    try {
      const dto: UpdateDollDto = {
        name,
        configuration: {
          colorScheme: {
            body: bodyColor,
            outline: outlineColor,
          },
        },
      };
      await invoke("update_doll", { id: editingDollId, dto });
      closeEditor();
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
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

  <DollEditor
    isOpen={isEditorOpen}
    mode={editorMode}
    initialName={editorInitialName}
    initialBodyColor={editorInitialBodyColor}
    initialOutlineColor={editorInitialOutlineColor}
    onSave={handleSave}
    onCancel={closeEditor}
  />
</div>
