<script lang="ts">
  import type { PresenceStatus } from "../../../types/bindings/PresenceStatus";
  import type { UserStatus } from "../../../events/user-status";

  interface Friend {
    friend?: {
      id: string;
      name: string;
    } | null;
  }

  interface Props {
    isInteractive: boolean;
    cursorPosition: { mapped: { x: number; y: number } };
    presenceStatus: PresenceStatus | null;
    friendsCursorPositions: Record<string, { mapped: { x: number; y: number } }>;
    friends: Friend[];
    friendsUserStatuses: Record<string, UserStatus>;
  }

  let {
    isInteractive,
    cursorPosition,
    presenceStatus,
    friendsCursorPositions,
    friends,
    friendsUserStatuses,
  }: Props = $props();

  function getFriendById(userId: string) {
    const friend = friends.find((f) => f.friend?.id === userId);
    return friend?.friend;
  }

  function getFriendStatus(userId: string) {
    return friendsUserStatuses[userId];
  }
</script>

<div
  class="size-max mx-auto bg-base-100 border-base-200 border p-1 rounded-lg shadow-md"
>
  <div class="flex flex-row gap-1 items-center text-center">
    <div>
      <span class="py-3 text-xs items-center gap-2 badge">
        <span
          class={`size-2 rounded-full ${isInteractive ? "bg-success" : "bg-base-300"}`}
        ></span>
        Interactive
      </span>
    </div>

    <span class="font-mono text-xs badge py-3">
      {cursorPosition.mapped.x.toFixed(3)}, {cursorPosition.mapped.y.toFixed(3)}
    </span>

    {#if presenceStatus}
      <span class="font-mono text-xs badge py-3 flex items-center gap-2">
        {#if presenceStatus.graphicsB64}
          <img
            src={`data:image/png;base64,${presenceStatus.graphicsB64}`}
            alt="Active app icon"
            class="size-4"
          />
        {/if}
        {presenceStatus.title}
      </span>
    {/if}

    {#if Object.keys(friendsCursorPositions).length > 0}
      <div class="flex flex-col gap-2">
        <div>
          {#each Object.entries(friendsCursorPositions) as [userId, position]}
            {@const status = getFriendStatus(userId)}
            <div class="badge py-3 text-xs text-left flex flex-row gap-2">
              <span class="font-bold">{getFriendById(userId)?.name}</span>
              <div class="flex flex-row font-mono gap-2">
                <span>
                  {position.mapped.x.toFixed(3)}, {position.mapped.y.toFixed(3)}
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
