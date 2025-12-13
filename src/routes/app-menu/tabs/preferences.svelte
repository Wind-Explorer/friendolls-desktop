<script>
  import { invoke } from "@tauri-apps/api/core";
  import { appData } from "../../../events/app-data";
  import Power from "../../../assets/icons/power.svelte";
</script>

<div class="size-full flex flex-col justify-between">
  <div>
    <p>{$appData?.user?.name}'s preferences</p>
  </div>
  <div class="w-full flex flex-row justify-between">
    <div></div>
    <div>
      <button
        class="btn btn-error btn-square btn-soft"
        onclick={async () => {
          await invoke("quit_app");
        }}
      >
        <div class="scale-50">
          <Power />
        </div>
      </button>
    </div>
  </div>
</div>
