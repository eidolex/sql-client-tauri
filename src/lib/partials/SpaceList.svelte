<script lang="ts">
  import * as ContextMenu from "$lib/components/ui/context-menu";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Button } from "$lib/components/ui/button";
  import { getAppState } from "$lib/stores/state.svelte";
  import { LogOut } from "lucide-svelte";
  import { cn } from "$lib/utils";
  import { disconnectDb } from "$lib/db";
  import { page } from "$app/state";

  const appState = getAppState();

  let isSpaceRoute = $derived.by(() => page.route.id == "/(space)/[id]");
  let paramsId = $derived.by(() => page.params.id);

  function getInitials(name: string) {
    return name
      .split(" ")
      .map((n) => n[0])
      .join("")
      .substring(0, 2)
      .toUpperCase();
  }

  async function closeConnection(connectionId: string) {
    try {
      await disconnectDb(connectionId);
      appState.removeSpace(connectionId);
    } catch (e) {
      console.error("Failed to disconnect", e);
    }
  }
</script>

<ScrollArea class="flex-1 w-full">
  <div class="flex flex-col items-center gap-3 w-full px-2">
    {#each appState.spaces.values() as space (space.id)}
      <ContextMenu.Root>
        <ContextMenu.Trigger>
          <Button
            variant={isSpaceRoute && paramsId === space.id
              ? "default"
              : "ghost"}
            size="icon"
            class={cn(
              "w-12 h-12 rounded-xl transition-all duration-200 relative group",
              isSpaceRoute && paramsId === space.id
                ? "bg-primary text-primary-foreground hover:bg-primary/90"
                : "bg-muted/50 hover:bg-muted"
            )}
            href={`/${space.id}`}
            title={space.config.name}
          >
            <span class="font-bold text-sm">
              {getInitials(space.config.name)}
            </span>

            <!-- Active Indicator -->
            {#if isSpaceRoute && paramsId === space.id}
              <div
                class="absolute -left-3 top-1/2 -translate-y-1/2 w-1 h-8 bg-foreground rounded-r-full"
              ></div>
            {/if}
          </Button>
        </ContextMenu.Trigger>
        <ContextMenu.Content>
          <ContextMenu.Item
            class="text-destructive focus:text-destructive"
            onclick={() => closeConnection(space.id)}
          >
            <LogOut class="mr-2 h-4 w-4" />
            Close Connection
          </ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Root>
    {/each}
  </div>
</ScrollArea>
