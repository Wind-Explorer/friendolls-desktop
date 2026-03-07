<script lang="ts">
  import { commands, type DollDto, type UserProfile } from "$lib/bindings";
  import { appData } from "../../../../events/app-data";
  import DollsList from "./dolls-list.svelte";

  let loading = false;
  let error: string | null = null;
  let user: UserProfile | null = null;
  let initialLoading = true;

  // Reactive - automatically updates when appData changes
  $: dolls = $appData?.dolls ?? [];
  $: user = $appData?.user ?? null;
  $: initialLoading = $appData === null;

  async function openCreateModal() {
    await commands.openDollEditorWindow(null);
  }

  async function openEditModal(doll: DollDto) {
    await commands.openDollEditorWindow(doll.id);
  }

  async function handleSetActiveDoll(dollId: string) {
    try {
      loading = true;
      await commands.setActiveDoll(dollId);
      // No manual refresh needed - backend will refresh and emit app-data-refreshed
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading = false;
    }
  }

  async function handleRemoveActiveDoll() {
    try {
      loading = true;
      await commands.removeActiveDoll();
      // No manual refresh needed - backend will refresh and emit app-data-refreshed
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="dolls-page flex flex-col gap-4 max-h-full h-full">
  <div class="flex justify-between items-center">
    <h2 class="text-lg font-bold">Your Nekos</h2>
    <button class="btn btn-primary btn-sm" on:click={openCreateModal}>
      Add a Neko
    </button>
  </div>

  <div class="overflow-y-auto rounded p-2 h-full border-base-200 border">
    <DollsList
      {dolls}
      {user}
      loading={loading || initialLoading}
      {error}
      onEditDoll={openEditModal}
      onSetActiveDoll={handleSetActiveDoll}
      onRemoveActiveDoll={handleRemoveActiveDoll}
    />
  </div>
</div>
