<script lang="ts">
  import { commands } from "$lib/bindings";
  import { appState } from "../../../../events/app-state";
  import DollPreview from "../../components/doll-preview.svelte";

  async function updateOpacity(value: number) {
    await commands.setSceneSetupNekosOpacity(value);
  }

  async function updateScale(value: number) {
    await commands.setSceneSetupNekosScale(value);
  }
</script>

<div class="collapse bg-base-100 border-base-300 border">
  <input type="checkbox" checked />
  <div class="collapse-title py-2 text-sm opacity-70">Neko View</div>
  <div class="collapse-content">
    <div class="pt-4 border-t border-base-300">
      <div class="flex flex-row gap-4">
        <div
          class="border border-primary relative shadow-[inset_0_0_8px] bg-primary/5 shadow-primary w-40 card"
        >
          <div class="size-full absolute bg-gridded opacity-25"></div>
          <div class="size-full absolute">
            <div
              class="flex flex-row size-full items-end justify-between text-[8px] text-primary p-1"
            >
              <div class="text-start flex flex-col">
                <p>Scale</p>
                <p>Opacity</p>
              </div>
              <div class="text-end flex flex-col">
                <p>{($appState.sceneSetup.nekosScale * 100).toFixed(0)}%</p>
                <p>{($appState.sceneSetup.nekosOpacity * 100).toFixed(0)}%</p>
              </div>
            </div>
          </div>
          <div
            class="size-full flex flex-row -translate-y-2 justify-center items-center"
          >
            <DollPreview
              dollColorScheme={{ body: "b7f2ff", outline: "496065" }}
              spriteScale={$appState.sceneSetup.nekosScale}
              spriteOpacity={$appState.sceneSetup.nekosOpacity}
            />
          </div>
        </div>
        <div class="flex flex-col gap-4 w-full">
          <div class="flex flex-col gap-2">
            <p class="text-xs opacity-70">Opacity</p>
            <div class="flex flex-row gap-2 items-center">
              <input
                type="range"
                class="range flex-1"
                min="0.1"
                max="1"
                step="0.01"
                value={$appState.sceneSetup.nekosOpacity}
                oninput={(event) =>
                  updateOpacity(
                    Number((event.currentTarget as HTMLInputElement).value),
                  )}
              />
            </div>
          </div>
          <div class="flex flex-col gap-2">
            <p class="text-xs opacity-70">Scale</p>
            <div class="flex flex-row gap-2 items-center">
              <input
                type="range"
                class="range flex-1"
                min="0.5"
                max="2"
                step="0.25"
                value={$appState.sceneSetup.nekosScale}
                oninput={(event) =>
                  updateScale(
                    Number((event.currentTarget as HTMLInputElement).value),
                  )}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .bg-gridded {
    background-image:
      linear-gradient(
        0deg,
        transparent 24%,
        var(--color-primary) 25%,
        var(--color-primary) 26%,
        transparent 27%,
        transparent 74%,
        var(--color-primary) 75%,
        var(--color-primary) 76%,
        transparent 77%,
        transparent
      ),
      linear-gradient(
        90deg,
        transparent 24%,
        var(--color-primary) 25%,
        var(--color-primary) 26%,
        transparent 27%,
        transparent 74%,
        var(--color-primary) 75%,
        var(--color-primary) 76%,
        transparent 77%,
        transparent
      );
    background-size: 32px 32px;
  }
</style>
