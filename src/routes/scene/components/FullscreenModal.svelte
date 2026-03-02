<script lang="ts">
  let {
    imageSrc,
    visible = $bindable(false),
    senderName = "",
  }: { imageSrc: string; visible: boolean; senderName?: string } = $props();

  let timer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    if (visible) {
      timer = setTimeout(() => {
        visible = false;
      }, 3000);
    }

    return () => {
      if (timer) {
        clearTimeout(timer);
        timer = null;
      }
    };
  });
</script>

{#if visible}
  <div class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-black/80">
    {#if senderName}
      <div class="mb-4 text-white text-lg font-medium">{senderName} gave you a headpat!</div>
    {/if}
    <img src={imageSrc} alt="Headpat" class="max-w-full max-h-full object-contain" />
  </div>
{/if}
