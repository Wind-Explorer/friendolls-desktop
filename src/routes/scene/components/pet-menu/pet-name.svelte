<script lang="ts">
  interface Props {
    name: string;
    visible?: boolean;
  }

  let { name, visible = true }: Props = $props();
</script>

{#if name}
  <div
    class={`absolute left-1/2 top-full z-10 mt-3 w-max max-w-32 -translate-x-1/2 rounded-md border border-base-300/80 bg-base-100/90 px-2 py-1 text-center text-[10px] font-medium leading-tight text-base-content/80 shadow-sm backdrop-blur-sm transition-all duration-200 ease-out ${
      visible
        ? "translate-y-0 opacity-100"
        : "pointer-events-none -translate-y-1 opacity-0"
    }`}
  >
    <span class="block truncate">{name}</span>
  </div>
{/if}
