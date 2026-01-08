<script lang="ts">
  import { type DollDto } from "../../../types/bindings/DollDto";
  import type { UserBasicDto } from "../../../types/bindings/UserBasicDto";
  export let doll: DollDto;
  export let user: UserBasicDto;
</script>

<div class="bg-base-100 border border-base-200 card p-1">
  <div class="flex flex-col gap-1">
    <div class="flex flex-row w-full items-end gap-1">
      <p class="text-sm font-semibold">{doll.name}</p>
      <p class="text-[0.6rem] opacity-50">From {user.name}</p>
    </div>
    <div class="flex flex-row gap-1 w-full *:flex-1 *:btn *:btn-sm">
      <button>Headpat</button>
      <button>Message</button>
    </div>
    <div class="flex flex-row gap-1 w-full *:flex-1 *:btn *:btn-sm">
      <button>Postcard</button>
      <button>Wormhole</button>
    </div>
  </div>
</div>
