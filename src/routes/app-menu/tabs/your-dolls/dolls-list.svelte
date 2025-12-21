<script lang="ts">
  import type { DollDto } from "../../../../types/bindings/DollDto";
  import type { UserProfile } from "../../../../types/bindings/UserProfile";
  import DollPreview from "../DollPreview.svelte";
  import PawPrint from "../../../../assets/icons/paw-print.svelte";
  import Backpack from "../../../../assets/icons/backpack.svelte";

  export let dolls: DollDto[];
  export let user: UserProfile | null;
  export let loading = false;
  export let error: string | null;
  export let onEditDoll: (doll: DollDto) => void;
  export let onSetActiveDoll: (dollId: string) => void;
  export let onRemoveActiveDoll: () => void;
</script>

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
              on:click={() => onEditDoll(doll)}
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
                  on:click={onRemoveActiveDoll}
                >
                  <div class="scale-60"><Backpack /></div>
                  Recall
                </button>
              {:else}
                <button
                  class="flex-1 text-primary"
                  on:click={() => onSetActiveDoll(doll.id)}
                >
                  <div class="scale-60"><PawPrint /></div>
                  Deploy
                </button>
              {/if}
            </div>
            <!-- <button
              class=" text-error"
              on:click={() => onDeleteDoll(doll.id)}
            >
              <div class="scale-60"><Trash /></div>
            </button> -->
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
