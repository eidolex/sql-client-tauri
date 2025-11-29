<script lang="ts">
  import { getAppState, type ActiveConnection } from "$lib/state.svelte";
  import { connectDb, disconnectDb, listTables } from "$lib/db";
  import {
    Database,
    Table,
    Terminal,
    LogOut,
    LoaderCircle,
  } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { cn } from "$lib/utils";
  import { page } from "$app/state";
  import { goto } from "$app/navigation";

  const appState = getAppState();

  // When the selected connection changes, we might want to reset or restore expanded state
  // For now, let's just keep it simple.

  let currentConnection = $derived(
    page.params.id ? appState.getConnection(page.params.id) : null
  );

  function openTable(connection: ActiveConnection, table: string) {
    // Check if tab already exists
    const existingTab = appState.tabs.find(
      (t) =>
        t.connectionId === connection.id &&
        t.table === table &&
        t.type === "data"
    );

    if (existingTab) {
      appState.activeTabId = existingTab.id;
    } else {
      appState.addTab({
        id: crypto.randomUUID(),
        title: table,
        type: "data",
        connectionId: connection.id,
        database: connection.currentDatabase,
        table: table,
      });
    }
  }

  function openSqlEditor(connection: ActiveConnection) {
    appState.addTab({
      id: crypto.randomUUID(),
      title: `Query: ${connection.config.name}`,
      type: "query",
      connectionId: connection.id,
      database: connection.currentDatabase,
    });
  }

  let connectingDatabases = $state<Set<string>>(new Set());

  async function switchDatabase(
    connection: ActiveConnection,
    database: string
  ) {
    // If we are already on this database, just toggle expansion
    if (connection.currentDatabase === database) {
      // It's already the active database for this connection
      return;
    }

    // Check if we are already connecting to this database
    const connectingKey = `${connection.id}-${database}`;
    if (connectingDatabases.has(connectingKey)) {
      return;
    }

    const existing = appState.activeConnections.find(
      (c) =>
        c.config.host === connection.config.host &&
        c.config.port == connection.config.port &&
        c.config.username === connection.config.username &&
        c.currentDatabase === database
    );

    if (existing) {
      appState.selectedConnectionId = existing.id;
      goto(`/${existing.id}`);
      return;
    }

    try {
      connectingDatabases.add(connectingKey);
      connectingDatabases = new Set(connectingDatabases);

      // Connect to new DB as a separate connection
      const newConfig = { ...connection.config, database };
      // Generate a temporary name for the new connection view
      newConfig.name = `${connection.config.name} (${database})`;

      const newId = await connectDb(newConfig);
      const tables = await listTables(newId);

      const newConnection = {
        id: newId,
        config: newConfig,
        databases: connection.databases, // Reuse the list of databases
        tables,
        currentDatabase: database,
      };

      appState.addConnection(newConnection);
      goto(`/${newId}`);
      // addConnection already sets selectedConnectionId
    } catch (e) {
      console.error("Failed to open database connection", e);
      alert("Failed to open database connection: " + e);
    } finally {
      connectingDatabases.delete(connectingKey);
      connectingDatabases = new Set(connectingDatabases);
    }
  }

  async function closeCurrentConnection() {
    if (currentConnection) {
      try {
        await disconnectDb(currentConnection.id);
        appState.removeConnection(currentConnection.id);
      } catch (e) {
        console.error("Failed to disconnect", e);
      }
    }
  }
</script>

<div class="w-64 bg-muted/30 h-full flex flex-col border-r">
  {#if !currentConnection}
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
        title={currentConnection.config.name}
      >
        {currentConnection.config.name}
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

        {#each currentConnection.databases as db}
          <div class="mb-1">
            <Button
              variant="ghost"
              class={cn(
                "w-full justify-start h-8 px-2 text-sm font-normal",
                currentConnection.currentDatabase === db
                  ? "text-primary font-medium bg-primary/10 hover:bg-primary/20"
                  : "text-muted-foreground"
              )}
              onclick={() => switchDatabase(currentConnection!, db)}
              disabled={connectingDatabases.has(
                `${currentConnection.id}-${db}`
              )}
            >
              {#if connectingDatabases.has(`${currentConnection.id}-${db}`)}
                <LoaderCircle class="mr-2 h-3.5 w-3.5 animate-spin" />
              {:else}
                <Database class="mr-2 h-3.5 w-3.5" />
              {/if}
              <span class="truncate flex-1 text-left">{db}</span>
              {#if currentConnection.currentDatabase === db}
                <div class="w-1.5 h-1.5 rounded-full bg-primary ml-auto"></div>
              {/if}
            </Button>

            <!-- Show tables if this is the active database -->
            {#if currentConnection.currentDatabase === db}
              <div class="ml-4 mt-1 border-l pl-2 space-y-0.5">
                <Button
                  variant="ghost"
                  class="w-full justify-start h-7 px-2 text-sm text-muted-foreground"
                  onclick={() => openSqlEditor(currentConnection!)}
                >
                  <Terminal class="mr-2 h-3.5 w-3.5" />
                  <span>SQL Editor</span>
                </Button>

                {#each currentConnection.tables as table}
                  <Button
                    variant="ghost"
                    class="w-full justify-start h-7 px-2 text-sm text-muted-foreground font-normal"
                    onclick={() => openTable(currentConnection!, table)}
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
