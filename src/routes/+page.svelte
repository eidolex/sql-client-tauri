<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import DataViewer from "../components/DataViewer.svelte";
  import StructureViewer from "../components/StructureViewer.svelte";
  import SqlEditor from "../components/SqlEditor.svelte";
  import Connection from "../components/Connection.svelte";
  import { X, Database, Terminal, TableProperties } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { cn } from "$lib/utils";

  function closeTab(id: string, e: Event) {
    e.stopPropagation();
    appState.closeTab(id);
  }

  function createNewQuery() {
    if (appState.selectedConnectionId) {
      const connection = appState.getConnection(appState.selectedConnectionId);
      if (connection) {
        appState.addTab({
          id: crypto.randomUUID(),
          type: "query",
          title: "New Query",
          connectionId: appState.selectedConnectionId,
          database: connection.currentDatabase,
          query: "",
        });
      }
    }
  }

  // Derived state for tabs of the currently selected connection
  let currentConnectionTabs = $derived(
    appState.selectedConnectionId
      ? appState.tabs.filter(
          (t) => t.connectionId === appState.selectedConnectionId,
        )
      : [],
  );

  let activeTabId = $derived(appState.activeTabId);

  function handleKeydown(e: KeyboardEvent) {
    // Check for Cmd+W (macOS) or Ctrl+W (Windows/Linux)
    if (e.key === "w" && (e.metaKey || e.ctrlKey)) {
      // Only close tab if there's an active tab
      if (activeTabId) {
        e.preventDefault(); // Prevent default browser behavior
        appState.closeTab(activeTabId);
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="h-full w-full flex flex-col bg-background">
  <!-- Second Level: Tab Bar (Only if a connection is selected) -->
  {#if appState.selectedConnectionId}
    <div class="flex bg-muted/10 border-b overflow-x-auto no-scrollbar">
      {#if currentConnectionTabs.length === 0}
        <div
          class="px-4 py-2 text-sm text-muted-foreground italic flex items-center h-10"
        >
          No open tabs
        </div>
      {/if}
      {#each currentConnectionTabs as tab (tab.id)}
        <button
          class={cn(
            "group px-4 py-2 text-sm font-medium border-r flex items-center gap-2 min-w-[140px] max-w-[240px] h-10 transition-colors relative",
            activeTabId === tab.id
              ? "bg-background text-foreground border-t-2 border-t-primary"
              : "bg-muted/10 text-muted-foreground hover:bg-muted/30 border-t-2 border-t-transparent",
          )}
          onclick={() => (appState.activeTabId = tab.id)}
        >
          {#if tab.type === "data"}
            <Database class="h-4 w-4 text-blue-500" />
          {:else if tab.type === "query"}
            <Terminal class="h-4 w-4 text-green-500" />
          {:else if tab.type === "structure"}
            <TableProperties class="h-4 w-4 text-orange-500" />
          {/if}
          <span class="truncate flex-1 text-left">{tab.title}</span>
          <div
            class={cn(
              "opacity-0 group-hover:opacity-100 rounded-sm p-0.5 hover:bg-destructive/10 hover:text-destructive transition-all",
              activeTabId === tab.id && "opacity-100",
            )}
            onclick={(e) => closeTab(tab.id, e)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && closeTab(tab.id, e)}
          >
            <X class="h-3 w-3" />
          </div>
        </button>
      {/each}
    </div>
  {/if}

  <!-- Content Area -->
  <div class="flex-1 overflow-hidden relative">
    {#if !appState.selectedConnectionId}
      <Connection />
    {:else}
      <!-- Show content for the selected connection's active tab -->
      {#if !activeTabId}
        <div
          class="flex flex-col items-center justify-center h-full text-muted-foreground gap-4"
        >
          <div class="bg-muted/30 p-6 rounded-full">
            <Database class="h-16 w-16 opacity-20" />
          </div>
          <div class="text-center space-y-2">
            <h3 class="text-lg font-semibold text-foreground">No Active Tab</h3>
            <p class="text-sm max-w-xs mx-auto">
              Select a table from the sidebar or start a new SQL query.
            </p>
          </div>
          <Button onclick={createNewQuery} variant="outline" class="mt-2">
            <Terminal class="mr-2 h-4 w-4" />
            New Query
          </Button>
        </div>
      {:else}
        {#each appState.tabs as tab (tab.id)}
          <div
            class="absolute inset-0 bg-background"
            class:hidden={activeTabId !== tab.id}
          >
            {#if tab.type === "data"}
              <DataViewer {tab} />
            {:else if tab.type === "structure"}
              <StructureViewer
                connectionId={tab.connectionId}
                tableName={tab.table!}
              />
            {:else if tab.type === "query"}
              <SqlEditor
                connectionId={tab.connectionId}
                initialQuery={tab.query}
              />
            {/if}
          </div>
        {/each}
      {/if}
    {/if}
  </div>
</div>
