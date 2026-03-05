<script lang="ts">
  import {
    cursorPositionOnScreen,
    friendsCursorPositions,
    friendsActiveDolls,
  } from "../../events/cursor";
  import { appData } from "../../events/app-data";
  import { sceneInteractive } from "../../events/scene-interactive";
  import {
    friendsUserStatuses,
    type UserStatus,
  } from "../../events/user-status";
  import { invoke } from "@tauri-apps/api/core";
  import DesktopPet from "./components/DesktopPet.svelte";
  import FullscreenModal from "./components/FullscreenModal.svelte";
  import { receivedInteractions, clearInteraction } from "$lib/stores/interaction-store";
  import { INTERACTION_TYPE_HEADPAT } from "$lib/constants/interaction";
  import { listen } from "@tauri-apps/api/event";
  import { getSpriteSheetUrl } from "$lib/utils/sprite-utils";
  import { onMount } from "svelte";
  import type { PresenceStatus } from "../../types/bindings/PresenceStatus";
  import type { DollDto } from "../../types/bindings/DollDto";

  let innerWidth = $state(0);
  let innerHeight = $state(0);

  let isInteractive = $derived($sceneInteractive);

  // Fullscreen modal state for headpats
  let showFullscreenModal = $state(false);
  let fullscreenImageSrc = $state("");
  let headpatSenderSpriteUrl = $state("");
  let headpatSenderId = $state<string | null>(null);
  let headpatTimer: ReturnType<typeof setTimeout> | null = null;

  // Queue for pending headpats (when modal is already showing)
  let headpatQueue = $state<Array<{ userId: string; content: string }>>([]);
  let headpatSpriteToken = 0;

  // Process next headpat in queue
  function processNextHeadpat() {
    if (headpatQueue.length > 0) {
      const next = headpatQueue.shift()!;
      clearInteraction(next.userId);
      headpatSenderId = next.userId;
      void loadHeadpatSprites(next.userId);
      showFullscreenModal = true;
      scheduleHeadpatDismiss();
    } else {
      fullscreenImageSrc = "";
      headpatSenderSpriteUrl = "";
      headpatSenderId = null;
    }
  }

  async function loadHeadpatSprites(senderId: string) {
    const token = ++headpatSpriteToken;
    const senderDoll = getFriendDoll(senderId);
    const userDoll = getUserDoll();

    let userPetpetGif = "";
    if (userDoll) {
      try {
        const gifBase64 = await invoke<string>("encode_pet_doll_gif_base64", { doll: userDoll });
        userPetpetGif = `data:image/gif;base64,${gifBase64}`;
      } catch (e) {
        console.error("Failed to generate user petpet:", e);
      }
    }

    const senderSpriteUrl = senderDoll
      ? await getSpriteSheetUrl({
          bodyColor: senderDoll.configuration.colorScheme.body,
          outlineColor: senderDoll.configuration.colorScheme.outline,
        })
      : await getSpriteSheetUrl();

    if (token !== headpatSpriteToken) return;
    fullscreenImageSrc = userPetpetGif;
    headpatSenderSpriteUrl = senderSpriteUrl;
  }

  function scheduleHeadpatDismiss() {
    if (headpatTimer) {
      clearTimeout(headpatTimer);
    }
    headpatTimer = setTimeout(() => {
      showFullscreenModal = false;
      headpatTimer = null;
    }, 3000);
  }

  function getHeadpatSenderName(userId: string | null): string {
    if (!userId) return "";
    const friend = getFriendById(userId);
    return friend?.name ?? "";
  }

  // Watch for headpat interactions and show fullscreen modal
  $effect(() => {
    for (const [userId, interaction] of $receivedInteractions) {
      if (interaction.type === INTERACTION_TYPE_HEADPAT) {
        if (showFullscreenModal) {
          // Queue the headpat for later (deduplicate by replacing existing from same user)
          const existingIndex = headpatQueue.findIndex((h) => h.userId === userId);
          if (existingIndex >= 0) {
            headpatQueue[existingIndex] = { userId, content: interaction.content };
          } else {
            headpatQueue.push({ userId, content: interaction.content });
          }
          scheduleHeadpatDismiss();
        } else {
          // Show immediately and clear from store
          clearInteraction(userId);
          headpatSenderId = userId;
          void loadHeadpatSprites(userId);
          showFullscreenModal = true;
          scheduleHeadpatDismiss();
        }
      }
    }
  });

  // When modal closes, process next headpat in queue
  $effect(() => {
    if (!showFullscreenModal && headpatSenderId) {
      headpatSenderId = null;
      processNextHeadpat();
    }
  });

  $effect(() => {
    return () => {
      if (headpatTimer) {
        clearTimeout(headpatTimer);
        headpatTimer = null;
      }
    };
  });

  function getFriendById(userId: string) {
    const friend = $appData?.friends?.find((f) => f.friend?.id === userId);
    return friend?.friend;
  }

  function getFriendDoll(userId: string) {
    if (userId in $friendsActiveDolls) {
      return $friendsActiveDolls[userId];
    }

    const friend = $appData?.friends?.find((f) => f.friend?.id === userId);
    return friend?.friend?.activeDoll;
  }

  function getFriendStatus(userId: string) {
    return $friendsUserStatuses[userId];
  }

  function getUserDoll(): DollDto | undefined {
    const user = $appData?.user;
    if (!user || !user.activeDollId) return undefined;
    return $appData?.dolls?.find((d) => d.id === user.activeDollId);
  }

  let presenceStatus: PresenceStatus | null = $state(null);

  onMount(() => {
    const unlisten = listen<UserStatus>("user-status-changed", (event) => {
      console.log("event received");
      presenceStatus = event.payload.presenceStatus;
    });

    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

<svelte:window bind:innerWidth bind:innerHeight />

<div class="w-svw h-svh p-4 relative overflow-hidden">
  <button
    class="absolute inset-0 z-10 size-full"
    aria-label="Deactive scene interactive"
    onmousedown={async () => {
      await invoke("set_scene_interactive", {
        interactive: false,
        shouldClick: true,
      });
    }}>&nbsp;</button
  >
  <div
    class="size-max mx-auto bg-base-100 border-base-200 border p-1 rounded-lg shadow-md"
  >
    <div class="flex flex-row gap-1 items-center text-center">
      <div>
        <span class="py-3 text-xs items-center gap-2 badge">
          <span
            class={`size-2 rounded-full ${isInteractive ? "bg-success" : "bg-base-300"}`}
          ></span>
          Intercepting cursor events
        </span>
      </div>

      <span class="font-mono text-xs badge py-3">
        ({$cursorPositionOnScreen.mapped.x.toFixed(3)}, {$cursorPositionOnScreen.mapped.y.toFixed(
          3,
        )})
      </span>

      <span class="font-mono text-xs badge py-3 flex items-center gap-2">
        {#if presenceStatus?.graphicsB64}
          <img
            src={`data:image/png;base64,${presenceStatus.graphicsB64}`}
            alt="Active app icon"
            class="size-4"
          />
        {/if}
        {presenceStatus?.title}
      </span>

      {#if Object.keys($friendsCursorPositions).length > 0}
        <div class="flex flex-col gap-2">
          <div>
            {#each Object.entries($friendsCursorPositions) as [userId, position]}
              {@const status = getFriendStatus(userId)}
              <div class="badge py-3 text-xs text-left flex flex-row gap-2">
                <span class="font-bold">{getFriendById(userId)?.name}</span>
                <div class="flex flex-row font-mono gap-2">
                  <span>
                    ({position.mapped.x.toFixed(3)}, {position.mapped.y.toFixed(
                      3,
                    )})
                  </span>
                  {#if status}
                    <span class="flex items-center gap-1">
                      {status.state} in
                      {#if status.presenceStatus.graphicsB64}
                        <img
                          src={`data:image/png;base64,${status.presenceStatus.graphicsB64}`}
                          alt="Friend's active app icon"
                          class="size-4"
                        />
                      {/if}
                      {status.presenceStatus.title ||
                        status.presenceStatus.subtitle}
                    </span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div class="absolute inset-0 size-full">
    {#if Object.keys($friendsCursorPositions).length > 0}
      {#each Object.entries($friendsCursorPositions) as [userId, position]}
        {@const doll = getFriendDoll(userId)}
        {@const friend = getFriendById(userId)}
        {#if doll && friend}
          <DesktopPet
            id={userId}
            targetX={position.mapped.x * innerWidth}
            targetY={position.mapped.y * innerHeight}
            user={friend}
            userStatus={getFriendStatus(userId)}
            {doll}
            {isInteractive}
            senderDoll={getUserDoll()}
          />
        {/if}
      {/each}
    {/if}
    {#if $appData?.user && getUserDoll()}
      <DesktopPet
        id={$appData.user.id}
        targetX={$cursorPositionOnScreen.mapped.x * innerWidth}
        targetY={$cursorPositionOnScreen.mapped.y * innerHeight}
        user={{
          id: $appData.user.id,
          name: $appData.user.name,
          username: $appData.user.username,
          activeDoll: getUserDoll() ?? null,
        }}
        userStatus={presenceStatus
          ? { presenceStatus: presenceStatus, state: "idle" }
          : undefined}
        doll={getUserDoll()}
        isInteractive={false}
      />
    {/if}
  </div>

  <FullscreenModal
    bind:visible={showFullscreenModal}
    imageSrc={fullscreenImageSrc}
    senderSpriteUrl={headpatSenderSpriteUrl}
    senderName={getHeadpatSenderName(headpatSenderId)}
  />
</div>
