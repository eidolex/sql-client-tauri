<script lang="ts">
  import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
  import { getConnectionState } from "$lib/stores/connection.state.svelte";
  import { cn } from "$lib/utils";
  import { Trash2 } from "lucide-svelte";
  import { onMount } from "svelte";

  const connectionState = getConnectionState();

  onMount(async () => {
    await connectionState.loadSavedConnections();
  });
</script>

<ScrollArea class="flex-1">
  <div class="p-2 space-y-1">
    {#each connectionState.connections as conn}
      <div
        class={cn(
          "group flex justify-between items-center p-2 rounded-md cursor-pointer transition-colors",
          connectionState.selectedConnectionId === conn.id
            ? "bg-primary/10 text-primary"
            : "hover:bg-muted"
        )}
        onclick={() => (connectionState.selectedConnectionId = conn.id)}
        onkeydown={(e) =>
          e.key === "Enter" && (connectionState.selectedConnectionId = conn.id)}
        role="button"
        tabindex="0"
      >
        <div class="truncate text-sm font-medium flex-1">
          {conn.name}
          <div class="text-xs text-muted-foreground truncate">
            {conn.username}@{conn.host}:{conn.port}
          </div>
        </div>
        <button
          type="button"
          class="h-6 w-6 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive inline-flex items-center justify-center rounded-md transition-colors hover:bg-accent"
          onclick={(e) => {
            e.stopPropagation();
            connectionState.remove(conn.id);
          }}
        >
          <Trash2 class="h-3 w-3" />
        </button>
      </div>
    {/each}
  </div>
</ScrollArea>
