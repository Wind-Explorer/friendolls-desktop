<script lang="ts">
  import type {
    NekoPositionDto,
    NekoPositionsDto,
    PresenceStatus,
    UserStatusPayload,
  } from "$lib/bindings";

  interface Friend {
    friend?: {
      id: string;
      name: string;
    } | null;
  }

  interface Props {
    isInteractive: boolean;
    nekoPositions: NekoPositionsDto;
    presenceStatus: PresenceStatus | null;
    friends: Friend[];
    friendsPresenceStates: Record<string, UserStatusPayload>;
  }

  let {
    isInteractive,
    nekoPositions,
    presenceStatus,
    friends,
    friendsPresenceStates,
  }: Props = $props();

  let selfCursor = $derived(
    Object.values(nekoPositions).find((position) => position?.isSelf)?.cursor,
  );

  let friendEntries = $derived.by(() => {
    return Object.entries(nekoPositions).filter(
      (entry): entry is [string, NekoPositionDto] => {
        return entry[1] !== undefined && !entry[1].isSelf;
      },
    );
  });

  function getFriendById(userId: string) {
    const friend = friends.find((f) => f.friend?.id === userId);
    return friend?.friend;
  }

  function getFriendStatus(userId: string) {
    return friendsPresenceStates[userId];
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

    {#if selfCursor}
      <span class="font-mono text-xs badge py-3">
        {selfCursor.mapped.x.toFixed(3)}, {selfCursor.mapped.y.toFixed(3)}
      </span>
    {/if}

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

    {#if friendEntries.length > 0}
      <div class="flex flex-col gap-2">
        <div>
          {#each friendEntries as [userId, position]}
            {@const status = getFriendStatus(userId)}
            <div class="badge py-3 text-xs text-left flex flex-row gap-2">
              <span class="font-bold">{getFriendById(userId)?.name}</span>
              <div class="flex flex-row font-mono gap-2">
                <span>
                  {position.cursor.mapped.x.toFixed(3)},
                  {position.cursor.mapped.y.toFixed(3)}
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
