<script lang="ts">
  import DollPreview from "../DollPreview.svelte";

  export let isOpen: boolean;
  export let mode: "create" | "edit";
  export let initialName = "";
  export let initialBodyColor = "#FFFFFF";
  export let initialOutlineColor = "#000000";
  export let onSave: (
    name: string,
    bodyColor: string,
    outlineColor: string,
  ) => void;
  export let onCancel: () => void;

  let name = initialName;
  let bodyColor = initialBodyColor;
  let outlineColor = initialOutlineColor;

  $: if (isOpen) {
    name = initialName;
    bodyColor = initialBodyColor;
    outlineColor = initialOutlineColor;
  }

  function handleSave() {
    if (!name.trim()) return;
    onSave(name, bodyColor, outlineColor);
  }
</script>

{#if isOpen}
  <div class="modal modal-open">
    <div class="modal-box">
      <h3 class="font-bold text-lg">
        {#if mode === "create"}
          Create New Doll
        {:else}
          Edit Doll
        {/if}
      </h3>
      <div class="form-control w-full mt-4">
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
      <div class="modal-action">
        <button class="btn" on:click={onCancel}>Cancel</button>
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
  </div>
{/if}
