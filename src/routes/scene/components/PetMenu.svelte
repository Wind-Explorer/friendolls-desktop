<script lang="ts">
  import { type DollConfigurationDto } from "../../../types/bindings/DollConfigurationDto";
  export let config: DollConfigurationDto;
</script>

<div class="bg-base-100 border border-base-200 card p-1">
  <div class="flex flex-col gap-1">
    <p class="text-xs">{config.colorScheme.body}</p>
  </div>
</div>
