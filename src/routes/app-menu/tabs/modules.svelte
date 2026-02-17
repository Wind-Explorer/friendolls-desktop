<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { ModuleMetadata } from "../../../types/bindings/ModuleMetadata";

  let modules: ModuleMetadata[] = [];
  let loading = false;
  let error: string | null = null;

  onMount(async () => {
    loading = true;
    try {
      modules = await invoke("get_modules");
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading = false;
    }
  });
</script>

<div class="modules-page flex flex-col gap-2">
  {#if error}
    <div class="text-error text-sm">{error}</div>
  {/if}

  <section class="flex flex-col gap-2">
    <div class="collapse bg-base-100 border-base-300 border">
      <input type="checkbox" checked />
      <div class="collapse-title text-sm opacity-70 py-2">
        Loaded Presence Modules
      </div>
      <div class="collapse-content px-2 -mb-2">
        <div class="flex flex-col gap-3">
          {#if loading}
            <p class="text-sm text-base-content/70">Loading modules...</p>
          {:else if modules.length === 0}
            <p class="text-sm text-base-content/70">No modules loaded.</p>
          {:else}
            <div class="flex flex-col gap-2">
              {#each modules as module (module.name)}
                <div class="card px-3 py-2 bg-base-200/50">
                  <div class="flex flex-col gap-1">
                    <div class="font-medium">{module.name}</div>
                    <div class="text-xs text-base-content/70">
                      Version: {module.version}
                    </div>
                    {#if module.description}
                      <div class="text-xs text-base-content/60">
                        {module.description}
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </section>
</div>
