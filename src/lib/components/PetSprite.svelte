<script lang="ts">
  export let spriteSheetUrl: string;
  export let spriteX: number;
  export let spriteY: number;
  export let size = 32;
</script>

<div
  class="pixelated"
  style="
    width: {size}px;
    height: {size}px;
    background-image: url('{spriteSheetUrl}');
    background-position: {spriteX}px {spriteY}px;
  "
></div>
