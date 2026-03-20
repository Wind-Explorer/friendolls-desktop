<script lang="ts">
  import { onDestroy } from "svelte";
  import { commands } from "$lib/bindings";
  import { sceneInteractive } from "../../../events/scene-interactive";
  import {
    closePetMessageSend,
    openPetMessageSendUserId,
  } from "../../../events/pet-message";
  import {
    createDocumentPointerHandler,
    createKeyDownHandler,
  } from "./pet-menu/events";
  import Send from "../../../assets/icons/send.svelte";

  interface Props {
    userId: string;
    userName: string;
  }

  const MAX_MESSAGE_LENGTH = 50;

  let { userId, userName }: Props = $props();

  let rootEl = $state<HTMLDivElement | null>(null);
  let messageText = $state("");
  let isSending = $state(false);

  const isOpen = $derived($openPetMessageSendUserId === userId);

  function closeMenu() {
    closePetMessageSend();
    messageText = "";
  }

  async function sendMessage() {
    const content = messageText.trim();
    if (!content || isSending || content.length > MAX_MESSAGE_LENGTH) {
      return;
    }

    isSending = true;
    try {
      await commands.sendInteractionCmd({
        recipientUserId: userId,
        content,
        type: "text",
      });
      closeMenu();
    } catch (error) {
      console.error("Failed to send interaction:", error);
    } finally {
      isSending = false;
    }
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
    if ($openPetMessageSendUserId === userId) {
      closePetMessageSend();
    }
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
  class={`absolute card bottom-9 flex flex-col left-4 z-40 w-56 border border-base-300 bg-base-100 p-2 text-base-content transition-all duration-200 ease-out ${
    isOpen && $sceneInteractive
      ? "pointer-events-auto opacity-100"
      : "pointer-events-none opacity-0"
  }`}
>
  <textarea
    class="h-20 p-1.5 outline-0 textarea textarea-xs bg-base-200 w-full resize-none"
    placeholder="Write a message..."
    maxlength={MAX_MESSAGE_LENGTH}
    bind:value={messageText}
  ></textarea>

  <div class="flex flex-row justify-between w-full items-center px-2">
    <div>
      <p class="text-[10px] text-base-content/70">
        {messageText.length}/{MAX_MESSAGE_LENGTH}
      </p>
    </div>
    <button
      type="button"
      class="btn btn-xs btn-primary rounded-t-none"
      disabled={isSending || !messageText.trim()}
      onclick={sendMessage}
    >
      <div class="*:size-2">
        <Send />
      </div>
      {isSending ? "Sending..." : "Send"}
    </button>
  </div>
</div>
