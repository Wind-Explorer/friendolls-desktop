<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { type DollDto } from "../../../types/bindings/DollDto";
  import type { UserBasicDto } from "../../../types/bindings/UserBasicDto";
  import type { SendInteractionDto } from "../../../types/bindings/SendInteractionDto";
  import type { FriendUserStatus } from "../../../events/user-status";

  export let doll: DollDto;
  export let user: UserBasicDto;
  export let userStatus: FriendUserStatus | undefined = undefined;
  export let receivedMessage: string | undefined = undefined;

  let showMessageInput = false;
  let messageContent = "";

  async function sendMessage() {
    if (!messageContent.trim()) return;

    const dto: SendInteractionDto = {
      recipientUserId: user.id,
      content: messageContent,
      type: "text",
    };

    try {
      await invoke("send_interaction_cmd", { dto });
      messageContent = "";
      showMessageInput = false;
    } catch (e) {
      console.error("Failed to send interaction:", e);
      alert("Failed to send message: " + e);
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      sendMessage();
    } else if (event.key === "Escape") {
      showMessageInput = false;
    }
  }
</script>

<div class="bg-base-100 border border-base-200 card p-1 min-w-[150px]">
  <div class="flex flex-col gap-1">
    <div class="flex flex-row w-full items-end gap-1">
      <p class="text-sm font-semibold">{doll.name}</p>
      <p class="text-[0.6rem] opacity-50">From {user.name}</p>
    </div>
    {#if userStatus}
      <div class="card bg-base-200 px-2 py-1 flex flex-row gap-2 items-center">
        {#if userStatus.appMetadata.appIconB64}
          <img
            src={`data:image/png;base64,${userStatus.appMetadata.appIconB64}`}
            alt="Friend's active app icon"
            class="size-3"
          />
        {/if}
        <p class="text-[0.6rem] font-mono text-ellipsis line-clamp-1">
          {userStatus.appMetadata.localized}
        </p>
      </div>
    {/if}

    {#if receivedMessage}
      <div class="">
        <div class="text-sm max-w-[140px]">
          {receivedMessage}
        </div>
      </div>
    {:else if showMessageInput}
      <div class="flex flex-col gap-1">
        <input
          type="text"
          bind:value={messageContent}
          onkeydown={handleKeydown}
          placeholder="Type message..."
          class="input input-xs input-bordered w-full"
        />
        <div class="flex gap-1">
          <button class="btn btn-xs btn-primary flex-1" onclick={sendMessage}
            >Send</button
          >
          <button
            class="btn btn-xs flex-1"
            onclick={() => (showMessageInput = false)}>Cancel</button
          >
        </div>
      </div>
    {:else}
      <div class="flex flex-row gap-1 w-full *:flex-1 *:btn *:btn-sm">
        <button disabled>Headpat</button>
        <button onclick={() => (showMessageInput = true)}>Message</button>
      </div>
      <div class="flex flex-row gap-1 w-full *:flex-1 *:btn *:btn-sm">
        <button disabled>Postcard</button>
        <button disabled>Wormhole</button>
      </div>
    {/if}
  </div>
</div>
