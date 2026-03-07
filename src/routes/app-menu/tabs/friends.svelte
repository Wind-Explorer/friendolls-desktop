<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import {
    commands,
    events,
    type FriendRequestResponseDto,
    type FriendshipResponseDto,
    type UserBasicDto,
  } from "$lib/bindings";
  import { appData } from "../../../events/app-data";

  let received: FriendRequestResponseDto[] = [];
  let sent: FriendRequestResponseDto[] = [];

  let loading = {
    received: false,
    sent: false,
    add: false,
    action: false,
  };

  let error: string | null = null;
  let searchTerm = "";

  let friends: FriendshipResponseDto[] = [];
  $: friends = $appData?.friends ?? [];

  type CombinedRequest = {
    id: string;
    type: "incoming" | "outgoing";
    request: FriendRequestResponseDto;
  };

  let combinedRequests: CombinedRequest[] = [];
  $: combinedRequests = [
    ...received.map((req) => ({
      id: `incoming-${req.id}`,
      type: "incoming" as const,
      request: req,
    })),
    ...sent.map((req) => ({
      id: `outgoing-${req.id}`,
      type: "outgoing" as const,
      request: req,
    })),
  ];

  let unlisteners: (() => void)[] = [];

  onMount(async () => {
    refreshReceived();
    refreshSent();

    unlisteners.push(
      await events.friendRequestReceived.listen(() => {
        refreshReceived();
      }),
    );

    unlisteners.push(
      await events.friendRequestAccepted.listen(() => {
        refreshSent();
        commands.refreshAppData();
      }),
    );

    unlisteners.push(
      await events.friendRequestDenied.listen(() => {
        refreshSent();
      }),
    );

    unlisteners.push(
      await events.unfriended.listen(() => {
        commands.refreshAppData();
      }),
    );
  });

  onDestroy(() => {
    unlisteners.forEach((unlisten) => unlisten());
  });

  async function refreshReceived() {
    loading.received = true;
    try {
      received = await commands.receivedFriendRequests();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.received = false;
    }
  }

  async function refreshSent() {
    loading.sent = true;
    try {
      sent = await commands.sentFriendRequests();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.sent = false;
    }
  }

  async function handleAccept(id: string) {
    loading.action = true;
    try {
      await commands.acceptFriendRequest(id);
      await Promise.all([refreshReceived(), commands.refreshAppData()]);
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.action = false;
    }
  }

  async function handleDeny(id: string) {
    loading.action = true;
    try {
      await commands.denyFriendRequest(id);
      await refreshReceived();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.action = false;
    }
  }

  async function handleUnfriend(friendId: string) {
    loading.action = true;
    try {
      await commands.unfriend(friendId);
      await commands.refreshAppData();
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.action = false;
    }
  }

  function clearSearch() {
    searchTerm = "";
    error = null;
  }

  async function handleAddFriend() {
    const term = searchTerm.trim();
    const sanitizedTerm = term.replace(/^@/, "");
    const normalizedTerm = sanitizedTerm.toLowerCase();

    if (!sanitizedTerm) {
      error = "Please enter a username.";
      return;
    }

    loading.add = true;
    error = null;

    try {
      const results = await commands.searchUsers(sanitizedTerm);
      const match = results.find(
        (user) => user.username?.toLowerCase() === normalizedTerm,
      );

      if (!match) {
        error = `No user found with username "${sanitizedTerm}".`;
        return;
      }

      await handleSendRequest(match.id);
      searchTerm = "";
    } catch (e) {
      error = (e as Error)?.message ?? String(e);
    } finally {
      loading.add = false;
    }
  }

  async function handleSendRequest(receiverId: string) {
    loading.action = true;
    try {
      await commands.sendFriendRequest({ receiverId });
      await refreshSent();
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      // If the server returns a conflict error (already friends or request exists), show "User not found"
      if (msg.toLowerCase().includes("already")) {
        error = "User not found";
      } else {
        error = msg;
      }
    } finally {
      loading.action = false;
    }
  }
</script>

<div class="friends-page flex flex-col gap-2">
  {#if error}
    <div class="text-error text-sm">{error}</div>
  {/if}

  <section class="flex flex-col gap-2">
    <div class="flex flex-col gap-3">
      <div class="flex flex-col gap-2">
        <div class="flex gap-2">
          <div class="relative flex-1 input input-bordered input-sm w-full">
            <input
              class="pr-20"
              placeholder="Add a friend"
              bind:value={searchTerm}
              on:keydown={(e) => e.key === "Enter" && handleAddFriend()}
            />
            {#if searchTerm.trim().length}
              <button
                type="button"
                class="btn btn-xs btn-ghost absolute right-1 top-1/2 -translate-y-1/2"
                on:click={clearSearch}
              >
                X
              </button>
            {/if}
          </div>
          <button
            class="btn btn-sm btn-primary"
            disabled={loading.add}
            on:click={handleAddFriend}
          >
            {loading.add ? "Adding..." : "Add"}
          </button>
        </div>
      </div>
    </div>
    <div class="collapse bg-base-100 border-base-300 border">
      <input type="checkbox" checked />
      <div class="collapse-title text-sm opacity-70 py-2">Friend requests</div>
      <div class="collapse-content px-2 -mb-2">
        <div class="flex flex-col gap-3">
          {#if loading.received || loading.sent}
            <p class="text-sm text-base-content/70">Loading requests...</p>
          {:else if combinedRequests.length === 0}
            <p class="text-sm text-base-content/70">
              No pending friend requests.
            </p>
          {:else}
            <div class="flex flex-col gap-2">
              {#each combinedRequests as entry (entry.id)}
                <div class="card px-3 py-2 bg-base-200/50">
                  <div class="flex items-center justify-between">
                    <div>
                      <div class="font-light">
                        {entry.type === "incoming"
                          ? entry.request.sender.name
                          : entry.request.receiver.name}
                      </div>
                      <div class="text-xs text-base-content/70">
                        @{entry.type === "incoming"
                          ? (entry.request.sender.username ?? "unknown")
                          : (entry.request.receiver.username ?? "unknown")}
                      </div>
                    </div>
                    {#if entry.type === "incoming"}
                      <div class="flex gap-2">
                        <button
                          class="btn btn-xs btn-primary"
                          disabled={loading.action}
                          on:click={() => handleAccept(entry.request.id)}
                        >
                          Accept
                        </button>
                        <button
                          class="btn btn-xs btn-ghost"
                          disabled={loading.action}
                          on:click={() => handleDeny(entry.request.id)}
                        >
                          Deny
                        </button>
                      </div>
                    {:else}
                      <div class="text-xs text-base-content/60 capitalize">
                        {entry.request.status}
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>

    <div class="collapse bg-base-100 border-base-300 border">
      <input type="checkbox" checked />
      <div class="collapse-title text-sm opacity-70 py-2">Friends</div>
      <div class="collapse-content px-2 -mb-2">
        <div class="flex flex-col gap-3">
          {#if friends.length === 0}
            <p class="text-sm text-base-content/70">No friends yet.</p>
          {:else}
            <div class="flex flex-col gap-2">
              {#each friends as friend (friend.id)}
                {#if friend.friend}
                  <div class="card px-3 py-2 bg-base-200/50">
                    <div class="flex items-center justify-between">
                      <div>
                        <div class="font-light">{friend.friend.name}</div>
                        <div class="text-xs text-base-content/70">
                          @{friend.friend.username ?? "unknown"}
                        </div>
                      </div>
                      <button
                        class="btn btn-sm btn-outline"
                        disabled={loading.action}
                        on:click={() => {
                          friend.friend
                            ? handleUnfriend(friend.friend.id)
                            : null;
                        }}
                      >
                        Unfriend
                      </button>
                    </div>
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  </section>
</div>
