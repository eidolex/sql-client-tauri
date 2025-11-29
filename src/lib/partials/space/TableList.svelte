<script lang="ts">
  import { getAppState, type ActiveConnection } from "$lib/stores/state.svelte";
  import { disconnectDb } from "$lib/db";
  import { Database, Table, Terminal, LogOut } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { cn } from "$lib/utils";
  import { goto } from "$app/navigation";

  let { spaceId }: { spaceId: string } = $props();

  const appState = getAppState();

  // When the selected connection changes, we might want to reset or restore expanded state
  // For now, let's just keep it simple.

  let currentSpace = $derived(spaceId ? appState.spaces.get(spaceId) : null);

  function openTable(table: string) {
    // Check if tab already exists
    const existingTab = appState.tabs
      .get(spaceId)
      ?.items.find(
        (t) =>
          (t.type === "data" || t.type === "structure") && t.table === table
      );

    if (existingTab) {
      appState.selectTab(spaceId, existingTab.id);
    } else {
      appState.addTab(spaceId, {
        id: crypto.randomUUID(),
        title: table,
        type: "data",
        connectionId: spaceId,
        database: currentSpace!.currentDatabase,
        table: table,
        page: 1,
        pageSize: 50,
        totalRows: 0,
      });
    }
  }

  function openSqlEditor(space: ActiveConnection) {
    appState.addTab(spaceId, {
      id: crypto.randomUUID(),
      title: `Query: ${space.config.name}`,
      type: "query",
      connectionId: space.id,
      database: space.currentDatabase,
    });
  }

  async function switchDatabase(space: ActiveConnection, database: string) {
    // If we are already on this database, just toggle expansion
    if (space.currentDatabase === database) {
      // It's already the active database for this connection
      return;
    }

    const newConnection = {
      ...space.config,
      database: database,
    };

    if (appState.hasConnection(newConnection)) {
      goto(`/${appState.getConnectionId(newConnection)}`);
      return;
    }

    const connectionId = await appState.addSpace(newConnection);
    goto(`/${connectionId}`);
  }

  async function closeCurrentConnection() {
    if (currentSpace) {
      try {
        await disconnectDb(currentSpace.id);
        appState.removeSpace(currentSpace.id);
      } catch (e) {
        console.error("Failed to disconnect", e);
      }
    }
  }
</script>

<div class="w-64 bg-muted/30 h-full flex flex-col border-r">
  {#if !currentSpace}
    <!-- Home / No Connection Selected -->
    <div class="p-4 border-b font-semibold flex items-center gap-2">
      <Database class="h-5 w-5" />
      <span>Explorer</span>
    </div>
    <div class="p-4 text-sm text-muted-foreground">
      Select a connection from the left bar or create a new one.
    </div>
  {:else}
    <!-- Active Connection Context -->
    <div class="p-4 border-b flex items-center justify-between group">
      <div
        class="font-semibold truncate text-sm"
        title={currentSpace.config.name}
      >
        {currentSpace.config.name}
      </div>
      <Button
        variant="ghost"
        size="icon"
        class="h-6 w-6 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive"
        onclick={closeCurrentConnection}
        title="Close Connection"
      >
        <LogOut class="h-3 w-3" />
      </Button>
    </div>

    <ScrollArea class="flex-1 min-h-0">
      <div class="p-2">
        <div
          class="text-xs text-muted-foreground uppercase font-semibold mb-2 px-2"
        >
          Databases
        </div>

        {#each currentSpace.databases as db}
          <div class="mb-1">
            <Button
              variant="ghost"
              class={cn(
                "w-full justify-start h-8 px-2 text-sm font-normal",
                currentSpace.currentDatabase === db
                  ? "text-primary font-medium bg-primary/10 hover:bg-primary/20"
                  : "text-muted-foreground"
              )}
              onclick={() => switchDatabase(currentSpace!, db)}
            >
              <!-- {#if connectingDatabases.has(`${currentSpace.id}-${db}`)}
                <LoaderCircle class="mr-2 h-3.5 w-3.5 animate-spin" />
              {:else} -->
              <Database class="mr-2 h-3.5 w-3.5" />
              <!-- {/if} -->
              <span class="truncate flex-1 text-left">{db}</span>
              {#if currentSpace.currentDatabase === db}
                <div class="w-1.5 h-1.5 rounded-full bg-primary ml-auto"></div>
              {/if}
            </Button>

            <!-- Show tables if this is the active database -->
            {#if currentSpace.currentDatabase === db}
              <div class="ml-4 mt-1 border-l pl-2 space-y-0.5">
                <Button
                  variant="ghost"
                  class="w-full justify-start h-7 px-2 text-sm text-muted-foreground"
                  onclick={() => openSqlEditor(currentSpace!)}
                >
                  <Terminal class="mr-2 h-3.5 w-3.5" />
                  <span>SQL Editor</span>
                </Button>

                {#each currentSpace.tables as table}
                  <Button
                    variant="ghost"
                    class="w-full justify-start h-7 px-2 text-sm text-muted-foreground font-normal"
                    onclick={() => openTable(table)}
                  >
                    <Table class="mr-2 h-3.5 w-3.5 text-muted-foreground/70" />
                    <span class="truncate">{table}</span>
                  </Button>
                {/each}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </ScrollArea>
  {/if}
</div>
