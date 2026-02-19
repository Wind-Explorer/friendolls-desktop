<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { usePetState } from "$lib/composables/usePetState";
  import { getSpriteSheetUrl } from "$lib/utils/sprite-utils";
  import PetSprite from "$lib/components/PetSprite.svelte";
  import onekoGif from "../../../assets/oneko/oneko.gif";
  import {
    receivedInteractions,
    clearInteraction,
  } from "$lib/stores/interaction-store";
  import PetMenu from "./PetMenu.svelte";
  import type { DollDto } from "../../../types/bindings/DollDto";
  import type { UserBasicDto } from "../../../types/bindings/UserBasicDto";
  import type { PresenceStatus } from "../../../types/bindings/PresenceStatus";
  import type { UserStatus } from "../../../events/user-status";
  import type { InteractionPayloadDto } from "../../../types/bindings/InteractionPayloadDto";

  export let id = "";
  export let targetX = 0;
  export let targetY = 0;
  export let user: UserBasicDto;
  export let userStatus: UserStatus | undefined = undefined;
  export let doll: DollDto | undefined = undefined;
  export let isInteractive = false;
  export let senderDoll: DollDto | undefined = undefined;

  const { position, currentSprite, updatePosition, setPosition } = usePetState(
    32,
    32,
  );

  let animationFrameId: number;
  let lastFrameTimestamp: number;
  let spriteSheetUrl = onekoGif;

  let isPetMenuOpen = false;
  let receivedInteraction: InteractionPayloadDto | undefined = undefined;
  let messageTimer: number | undefined = undefined;

  // Watch for received interactions for this user
  $: {
    const interaction = $receivedInteractions.get(user.id);
    if (interaction && interaction !== receivedInteraction) {
      receivedInteraction = interaction;
      isPetMenuOpen = true;

      // Make scene interactive so user can see it
      invoke("set_scene_interactive", {
        interactive: true,
        shouldClick: false,
      });

      // Clear existing timer if any
      if (messageTimer) clearTimeout(messageTimer);

      // Auto-close and clear after 8 seconds
      messageTimer = setTimeout(() => {
        isPetMenuOpen = false;
        receivedInteraction = undefined;
        clearInteraction(user.id);
        // We probably shouldn't disable interactivity globally here as other pets might be active,
        // but 'set_pet_menu_state' in backend handles the window transparency logic per pet/menu.
        // However, we did explicitly call set_scene_interactive(true).
        // It might be safer to let the mouse-leave or other logic handle setting it back to false,
        // or just leave it as is since the user might want to interact.
        // For now, focusing on the message lifecycle.
      }, 8000) as unknown as number;
    }
  }

  // Watch for color changes to regenerate sprite
  $: updateSprite(
    doll?.configuration.colorScheme.body,
    doll?.configuration.colorScheme.outline,
  );

  // This reactive statement forces the menu closed whenever `isInteractive` changes.
  // This conflicts with our message logic because we explicitly set interactive=true when opening the menu for a message.
  // We should remove this or condition it.
  // The original intent was likely to close the menu if the user moves the mouse away (interactive becomes false),
  // but `isInteractive` is driven by mouse hover usually.
  // When we force it via invoke("set_scene_interactive", { interactive: true }), it might not reflect back into `isInteractive` prop immediately or correctly depending on how the parent passes it.
  // Actually, `isInteractive` is a prop passed from +page.svelte probably based on hover state.
  // If we want the menu to stay open during the message, we should probably ignore this auto-close behavior if a message is present.

  $: if (!receivedInteraction && !isInteractive) {
    isPetMenuOpen = false;
  }

  $: {
    if (id) {
      console.log(`Setting pet menu state for ${id}: ${isPetMenuOpen}`);
      invoke("set_pet_menu_state", { id, open: isPetMenuOpen });
    }
  }

  async function updateSprite(
    body: string | undefined,
    outline: string | undefined,
  ) {
    if (body && outline) {
      spriteSheetUrl = await getSpriteSheetUrl({
        bodyColor: body,
        outlineColor: outline,
      });
    } else {
      spriteSheetUrl = onekoGif;
    }
  }

  function frame(timestamp: number) {
    if (!lastFrameTimestamp) {
      lastFrameTimestamp = timestamp;
    }

    // 100ms per frame for the animation loop
    if (timestamp - lastFrameTimestamp > 100) {
      lastFrameTimestamp = timestamp;
      if (!isPetMenuOpen) {
        updatePosition(targetX, targetY, window.innerWidth, window.innerHeight);
      }
    }

    animationFrameId = requestAnimationFrame(frame);
  }

  onMount(() => {
    // Initialize position to target so it doesn't fly in from 32,32 every time
    setPosition(targetX, targetY);
    animationFrameId = requestAnimationFrame(frame);
  });

  onDestroy(() => {
    if (animationFrameId) {
      cancelAnimationFrame(animationFrameId);
    }
  });
</script>

<div
  class="desktop-pet flex flex-col items-center relative"
  style="
    transform: translate({$position.x - 16}px, {$position.y - 16}px);
    z-index: 50;
  "
>
  {#if isPetMenuOpen}
    <div
      class="z-10 absolute -translate-y-30 w-50 h-28 *:size-full shadow-md rounded"
      role="menu"
      tabindex="0"
      aria-label="Pet Menu"
    >
      {#if doll}
        <PetMenu
          {doll}
          {user}
          {userStatus}
          {receivedInteraction}
          {senderDoll}
        />
      {/if}
    </div>
  {/if}
  {#if userStatus}
    <div class="absolute -top-5 left-0 right-0 w-max mx-auto">
      {#if userStatus.presenceStatus.graphicsB64}
        <img
          src={`data:image/png;base64,${userStatus.presenceStatus.graphicsB64}`}
          alt="Friend's active app icon"
          class="size-4"
        />
      {/if}
    </div>
  {/if}
  <button
    disabled={!isInteractive}
    onclick={() => {
      if (!isInteractive) return;
      isPetMenuOpen = !isPetMenuOpen;
      if (!isPetMenuOpen) {
        // Clear message when closing menu manually
        receivedInteraction = undefined;
        clearInteraction(user.id);
        if (messageTimer) clearTimeout(messageTimer);
      }
    }}
  >
    <PetSprite
      {spriteSheetUrl}
      spriteX={$currentSprite.x}
      spriteY={$currentSprite.y}
    />
  </button>

  <span
    class="absolute -bottom-5 width-full text-[10px] bg-black/50 text-white px-1 rounded backdrop-blur-sm mt-1 whitespace-nowrap opacity-0 transition-opacity"
    class:opacity-100={isInteractive && !isPetMenuOpen}
  >
    {doll?.name}
  </span>
</div>

<style>
  .desktop-pet {
    position: fixed; /* Fixed relative to the viewport/container */
    top: 0;
    left: 0;
    will-change: transform;
  }
</style>
