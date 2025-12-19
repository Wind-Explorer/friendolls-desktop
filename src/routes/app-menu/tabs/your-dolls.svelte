<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { DollDto } from "../../../types/bindings/DollDto";
  import type { CreateDollDto } from "../../../types/bindings/CreateDollDto";
  import type { UpdateDollDto } from "../../../types/bindings/UpdateDollDto";

  let dolls: DollDto[] = [];
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
</script>

<div class="dolls-page flex flex-col gap-4 p-4">
  <div class="flex justify-between items-center">
    <h2 class="text-xl font-bold">Your Dolls</h2>
    <button class="btn btn-primary btn-sm" on:click={openCreateModal}>
      Create Doll
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

  {#if loading}
    <div class="flex justify-center p-4">
      <span class="loading loading-spinner loading-md"></span>
    </div>
  {:else if dolls.length === 0}
    <div class="text-center text-base-content/70 py-8">
      <p>No dolls found. Create your first doll!</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each dolls as doll (doll.id)}
        <div class="card bg-base-200 shadow-sm">
          <div class="card-body p-4">
            <h3 class="card-title text-base">{doll.name}</h3>
            <div class="flex gap-2 text-sm text-base-content/70">
              <div class="flex items-center gap-1">
                <div
                  class="w-4 h-4 rounded border border-base-content/20"
                  style="background-color: {doll.configuration.colorScheme
                    .body};"
                ></div>
                <span>Body</span>
              </div>
              <div class="flex items-center gap-1">
                <div
                  class="w-4 h-4 rounded border border-base-content/20"
                  style="background-color: {doll.configuration.colorScheme
                    .outline};"
                ></div>
                <span>Outline</span>
              </div>
            </div>
            <div class="card-actions justify-end mt-2">
              <button
                class="btn btn-xs btn-ghost"
                on:click={() => openEditModal(doll)}>Edit</button
              >
              <button
                class="btn btn-xs btn-ghost text-error"
                on:click={() => handleDeleteDoll(doll.id)}>Delete</button
              >
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}

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
