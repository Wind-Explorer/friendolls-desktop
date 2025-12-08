<script>
  import { invoke } from "@tauri-apps/api/core";
  import { appData } from "../../../events/app-data";
</script>

<div class="size-full flex flex-col justify-between">
  <div>
    <p>{$appData?.user?.name}'s preferences</p>
  </div>
  <div class="w-full flex flex-row justify-between">
    <div></div>
    <div>
      <button
        class="btn btn-error btn-sm btn-soft"
        onclick={async () => {
          await invoke("quit_app");
        }}>Quit Friendolls</button
      >
    </div>
  </div>
</div>
