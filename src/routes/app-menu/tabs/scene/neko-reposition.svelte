<script lang="ts">
  import { commands } from "$lib/bindings";
  import { appState, type NeksPosition } from "../../../../events/app-state";

  const positions: { value: NeksPosition | null; label: string }[] = [
    { value: "top-left", label: "Top Left" },
    { value: "top", label: "Top" },
    { value: "top-right", label: "Top Right" },
    { value: "left", label: "Left" },
    { value: null, label: "" },
    { value: "right", label: "Right" },
    { value: "bottom-left", label: "Bottom Left" },
    { value: "bottom", label: "Bottom" },
    { value: "bottom-right", label: "Bottom Right" },
  ];

  async function selectPosition(position: NeksPosition | null) {
    await commands.setSceneSetupNekosPosition(
      $appState.sceneSetup.nekosPosition === position ? null : position,
    );
  }

  let selectedLabel = $derived(
    positions.find((p) => p.value === $appState.sceneSetup.nekosPosition)
      ?.label ?? "",
  );
</script>

<div class="collapse bg-base-100 border-base-300 border">
  <input type="checkbox" checked />
  <div class="collapse-title py-2 text-sm opacity-70">Neko Reposition</div>
  <div class="collapse-content">
    <div class="flex flex-row gap-4 h-full pt-4 border-t border-base-300">
      <div class="h-full flex flex-col justify-between">
        <div>
          <p class="text-sm opacity-50">
            Choose a corner to gather nekos into a cluster
          </p>
        </div>
        <div>
          <p class="text-sm">
            {$appState.sceneSetup.nekosPosition
              ? selectedLabel
              : "Click a corner to enable"}
          </p>
        </div>
      </div>
      <div class="card bg-base-200/50 p-1 w-max border border-base-300">
        <div class="grid grid-cols-3 gap-6 items-center w-max">
          {#each positions as pos}
            {#if pos.value === null}
              <div></div>
            {:else}
              <button
                class={"btn-xs btn btn-square " +
                  ($appState.sceneSetup.nekosPosition === pos.value
                    ? "btn-primary"
                    : "")}
                aria-label={pos.label}
                onclick={() => selectPosition(pos.value)}
              ></button>
            {/if}
          {/each}
        </div>
      </div>
    </div>
  </div>
</div>
