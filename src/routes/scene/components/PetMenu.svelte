<script lang="ts">
  import { type DollDto } from "../../../types/bindings/DollDto";
  export let doll: DollDto;
</script>

<div class="bg-base-100 border border-base-200 card p-1">
  <div class="flex flex-col gap-1">
    <p class="text-xs">{doll.name}</p>
  </div>
</div>
