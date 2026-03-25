<script lang="ts">
  import { onDestroy } from "svelte";
  import { getButtonPosition } from "./position";
  import {
    createDocumentPointerHandler,
    createKeyDownHandler,
    createPetActions,
  } from "./events";
  import { sceneInteractive } from "../../../../events/scene-interactive";
  import { commands, type UserBasicDto } from "$lib/bindings";
  import PetName from "./pet-name.svelte";

  export interface PetMenuAction {
    icon: string;
    label: string;
    onClick?: () => void;
  }

  interface Props {
    user?: UserBasicDto;
    ariaLabel?: string;
  }

  let { user, ariaLabel = "Toggle pet actions" }: Props = $props();

  const actions = $derived(user ? createPetActions(user) : []);
  const dollName = $derived(user?.activeDoll?.name ?? "");

  let rootEl = $state<HTMLDivElement | null>(null);
  let isOpen = $state(false);

  function closeMenu() {
    if (!user) return;
    isOpen = false;
    commands.setPetMenuState(user.id, false);
  }

  function toggleMenu() {
    if (!user) return;
    if (!$sceneInteractive || actions.length === 0) {
      commands.setPetMenuState(user.id, false);
      closeMenu();
      return;
    }

    isOpen = !isOpen;
    commands.setPetMenuState(user.id, isOpen);
  }

  function handleActionClick(action: PetMenuAction) {
    if (!$sceneInteractive) {
      return;
    }

    action.onClick?.();
    closeMenu();
  }

  const handleDocumentPointerDown = createDocumentPointerHandler(
    () => isOpen,
    () => rootEl,
    closeMenu,
  );

  const handleKeyDown = createKeyDownHandler(() => isOpen, closeMenu);

  $effect(() => {
    if (!$sceneInteractive && isOpen) {
      closeMenu();
    }
  });

  $effect(() => {
    if (isOpen) {
      document.addEventListener("pointerdown", handleDocumentPointerDown, true);
      document.addEventListener("keydown", handleKeyDown, true);
    }

    return () => {
      document.removeEventListener(
        "pointerdown",
        handleDocumentPointerDown,
        true,
      );
      document.removeEventListener("keydown", handleKeyDown, true);
    };
  });

  onDestroy(() => {
    document.removeEventListener(
      "pointerdown",
      handleDocumentPointerDown,
      true,
    );
    document.removeEventListener("keydown", handleKeyDown, true);
  });
</script>

<div
  bind:this={rootEl}
  class="pointer-events-auto absolute inset-0 overflow-visible"
>
  {#each actions as action, index}
    {@const position = getButtonPosition(index, actions.length)}
    {@const openDelay = index * 35}
    {@const closeDelay = (actions.length - 1 - index) * 25}

    <button
      type="button"
      class={`absolute left-8 top-8 z-20 flex size-8 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full border border-base-300/80 bg-base-100/95 text-base-content shadow-md backdrop-blur-sm transition-[opacity,transform] duration-200 ease-out focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/60 ${
        isOpen && $sceneInteractive
          ? "opacity-100 hover:cursor-pointer"
          : "pointer-events-none opacity-0"
      }`}
      style={`transform: translate(calc(-50% + ${position.x}px), calc(-50% + ${position.y}px)) scale(${isOpen && $sceneInteractive ? 1 : 0.72}); transition-delay: ${isOpen && $sceneInteractive ? openDelay : closeDelay}ms;`}
      aria-label={action.label}
      title={action.label}
      onclick={() => {
        handleActionClick(action);
      }}
    >
      <span class="text-[11px] leading-none">{action.icon}</span>
    </button>
  {/each}

  {#if dollName}
    <PetName name={dollName} visible={$sceneInteractive} />
  {/if}

  <button
    type="button"
    class={`absolute inset-0 z-30 transition-all duration-200 ease-out focus:outline-none ${
      $sceneInteractive
        ? "cursor-pointer"
        : "pointer-events-none cursor-default"
    } ${isOpen ? "ring-0" : ""}`}
    aria-expanded={isOpen}
    aria-label={ariaLabel}
    tabindex={$sceneInteractive ? 0 : -1}
    onclick={toggleMenu}
    onkeydown={(e) => {
      if (e.key === "Enter" || e.key === " ") {
        toggleMenu();
        e.preventDefault();
      }
    }}
  >
    <span class="sr-only">{ariaLabel}</span>
  </button>
</div>
