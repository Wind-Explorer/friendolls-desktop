<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { DollDto } from "../../../types/bindings/DollDto";
  import type { UserProfile } from "../../../types/bindings/UserProfile";
  import type { AppData } from "../../../types/bindings/AppData";
  import type { CreateDollDto } from "../../../types/bindings/CreateDollDto";
  import type { UpdateDollDto } from "../../../types/bindings/UpdateDollDto";
  import DollPreview from "./DollPreview.svelte";
  import PawPrint from "../../../assets/icons/paw-print.svelte";
  import Backpack from "../../../assets/icons/backpack.svelte";

  let dolls: DollDto[] = [];
  let user: UserProfile | null = null;
  let loading = false;
  let error: string | null = null;
  let isCreateModalOpen = false;
  let isEditModalOpen = false;
  let editingDollId: string | null = null;

  let newDollName = "";
  let newDollColorBody = "#FFFFFF";
  let newDollColorOutline = "#000000";

  let editDollName = "";
  let editDollColorBody = "#FFFFFF";
  let editDollColorOutline = "#000000";

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
    newDollName = "";
    newDollColorBody = "#FFFFFF";
    newDollColorOutline = "#000000";
    isCreateModalOpen = true;
  }

  function closeCreateModal() {
    isCreateModalOpen = false;
  }

  async function handleCreateDoll() {
    if (!newDollName.trim()) return;

    try {
      const dto: CreateDollDto = {
        name: newDollName,
        configuration: {
          colorScheme: {
            body: newDollColorBody,
            outline: newDollColorOutline,
          },
        },
      };
      await invoke("create_doll", { dto });
      closeCreateModal();
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  function openEditModal(doll: DollDto) {
    editingDollId = doll.id;
    editDollName = doll.name;
    editDollColorBody = doll.configuration.colorScheme.body;
    editDollColorOutline = doll.configuration.colorScheme.outline;
    isEditModalOpen = true;
  }

  function closeEditModal() {
    isEditModalOpen = false;
    editingDollId = null;
  }

  async function handleUpdateDoll() {
    if (!editingDollId || !editDollName.trim()) return;

    try {
      const dto: UpdateDollDto = {
        name: editDollName,
        configuration: {
          colorScheme: {
            body: editDollColorBody,
            outline: editDollColorOutline,
          },
        },
      };
      await invoke("update_doll", { id: editingDollId, dto });
      closeEditModal();
      await refreshDolls();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    }
  }

  async function handleDeleteDoll(id: string) {
    if (!confirm("Are you sure you want to delete this doll?")) return;

    try {
      await invoke("delete_doll", { id });
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

  {#if error}
    <div class="alert alert-error">
      <span>{error}</span>
      <button class="btn btn-xs btn-ghost" on:click={() => (error = null)}
        >X</button
      >
    </div>
  {/if}

  <div class="flex flex-col relative">
    {#if loading}
      <progress class="progress w-full h-px absolute inset-0 z-10"></progress>
    {/if}

    {#if dolls.length === 0}
      <div class="text-center text-base-content/70 py-8">
        <p>No dolls found. Create your first doll!</p>
      </div>
    {:else}
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
        {#each dolls as doll (doll.id)}
          <div
            class="card border border-base-200 bg-linear-to-b from-base-100 to-base-300 relative"
          >
            <div class="flex flex-col w-full">
              <button
                on:click={() => openEditModal(doll)}
                class="flex flex-col w-full text-center py-6 gap-2 *:mx-auto hover:opacity-70 hover:cursor-pointer"
              >
                <div class="flex justify-center">
                  <DollPreview
                    bodyColor={doll.configuration.colorScheme.body}
                    outlineColor={doll.configuration.colorScheme.outline}
                  />
                </div>
                <p
                  style:background-color={doll.configuration.colorScheme.body}
                  style:color={doll.configuration.colorScheme.outline}
                  class="badge border-none text-xs w-max mx-auto"
                >
                  {doll.name}
                </p>
              </button>

              <div class="*:btn *:btn-block *:rounded-t-none">
                {#if user?.activeDollId === doll.id}
                  <button
                    class="btn-primary text-accent flex-1"
                    on:click={handleRemoveActiveDoll}
                  >
                    <div class="scale-60"><Backpack /></div>
                    Recall
                  </button>
                {:else}
                  <button
                    class="flex-1 text-primary"
                    on:click={() => handleSetActiveDoll(doll.id)}
                  >
                    <div class="scale-60"><PawPrint /></div>
                    Deploy
                  </button>
                {/if}
              </div>
              <!-- <button
                class=" text-error"
                on:click={() => handleDeleteDoll(doll.id)}
              >
                <div class="scale-60"><Trash /></div>
              </button> -->
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Create Modal -->
  {#if isCreateModalOpen}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">Create New Doll</h3>
        <div class="form-control w-full mt-4">
          <label class="label">
            <span class="label-text">Name</span>
          </label>
          <input
            type="text"
            placeholder="Doll Name"
            class="input input-bordered w-full"
            bind:value={newDollName}
          />
        </div>
        <div class="flex justify-center mt-4">
          <DollPreview
            bodyColor={newDollColorBody}
            outlineColor={newDollColorOutline}
          />
        </div>
        <div class="form-control w-full mt-2">
          <label class="label">
            <span class="label-text">Body Color</span>
          </label>
          <div class="flex gap-2">
            <input
              type="color"
              class="input input-bordered w-12 p-1 h-10"
              bind:value={newDollColorBody}
            />
            <input
              type="text"
              class="input input-bordered w-full"
              bind:value={newDollColorBody}
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
              bind:value={newDollColorOutline}
            />
            <input
              type="text"
              class="input input-bordered w-full"
              bind:value={newDollColorOutline}
            />
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" on:click={closeCreateModal}>Cancel</button>
          <button
            class="btn btn-primary"
            on:click={handleCreateDoll}
            disabled={!newDollName.trim()}>Create</button
          >
        </div>
      </div>
    </div>
  {/if}

  <!-- Edit Modal -->
  {#if isEditModalOpen}
    <div class="modal modal-open">
      <div class="modal-box">
        <h3 class="font-bold text-lg">Edit Doll</h3>
        <div class="form-control w-full mt-4">
          <label class="label">
            <span class="label-text">Name</span>
          </label>
          <input
            type="text"
            placeholder="Doll Name"
            class="input input-bordered w-full"
            bind:value={editDollName}
          />
        </div>
        <div class="flex justify-center mt-4">
          <DollPreview
            bodyColor={editDollColorBody}
            outlineColor={editDollColorOutline}
          />
        </div>
        <div class="form-control w-full mt-2">
          <label class="label">
            <span class="label-text">Body Color</span>
          </label>
          <div class="flex gap-2">
            <input
              type="color"
              class="input input-bordered w-12 p-1 h-10"
              bind:value={editDollColorBody}
            />
            <input
              type="text"
              class="input input-bordered w-full"
              bind:value={editDollColorBody}
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
              bind:value={editDollColorOutline}
            />
            <input
              type="text"
              class="input input-bordered w-full"
              bind:value={editDollColorOutline}
            />
          </div>
        </div>
        <div class="modal-action">
          <button class="btn" on:click={closeEditModal}>Cancel</button>
          <button
            class="btn btn-primary"
            on:click={handleUpdateDoll}
            disabled={!editDollName.trim()}>Save</button
          >
        </div>
      </div>
    </div>
  {/if}
</div>
