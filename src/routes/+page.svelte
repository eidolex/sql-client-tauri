<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import DataViewer from "../components/DataViewer.svelte";
  import StructureViewer from "../components/StructureViewer.svelte";
  import SqlEditor from "../components/SqlEditor.svelte";
  import Connection from "../components/Connection.svelte";
  import { X, Database } from "lucide-svelte";

  function closeTab(id: string, e: Event) {
    e.stopPropagation();
    appState.closeTab(id);
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
</script>

<div class="h-full w-full flex flex-col bg-gray-950">
  <!-- Second Level: Tab Bar (Only if a connection is selected) -->
  {#if appState.selectedConnectionId}
    <div class="flex bg-gray-900 border-b border-gray-800 overflow-x-auto pl-2">
      {#if currentConnectionTabs.length === 0}
        <div class="px-4 py-2 text-sm text-gray-500 italic">
          No open tabs. Select a table from the sidebar.
        </div>
      {/if}
      {#each currentConnectionTabs as tab (tab.id)}
        <button
          class="group px-4 py-2 text-sm font-medium border-r border-gray-800 hover:bg-gray-800 flex items-center gap-2 min-w-[120px] max-w-[200px]"
          class:bg-gray-800={activeTabId === tab.id}
          class:text-white={activeTabId === tab.id}
          class:text-gray-400={activeTabId !== tab.id}
          onclick={() => (appState.activeTabId = tab.id)}
        >
          {#if tab.type === "data"}
            <Database size={14} class="text-blue-400" />
          {:else if tab.type === "query"}
            <span class="text-xs font-mono bg-gray-700 px-1 rounded">SQL</span>
          {/if}
          <span class="truncate flex-1 text-left">{tab.title}</span>
          <span
            class="opacity-0 group-hover:opacity-100 hover:text-red-400 rounded p-0.5"
            onclick={(e) => closeTab(tab.id, e)}
            role="button"
            tabindex="0"
            onkeydown={(e) => e.key === "Enter" && closeTab(tab.id, e)}
          >
            <X size={14} />
          </span>
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
          class="flex flex-col items-center justify-center h-full text-gray-500"
        >
          <Database size={48} class="mb-4 opacity-20" />
          <p>Select a table or run a query</p>
        </div>
      {:else}
        {#each appState.tabs as tab (tab.id)}
          <div
            class="absolute inset-0 bg-gray-950"
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
