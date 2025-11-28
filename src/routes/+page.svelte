<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import DataViewer from "../components/DataViewer.svelte";
  import StructureViewer from "../components/StructureViewer.svelte";
  import SqlEditor from "../components/SqlEditor.svelte";
  import Connection from "../components/Connection.svelte";
  import { X } from "lucide-svelte";

  function closeTab(id: string, e: Event) {
    e.stopPropagation();
    appState.closeTab(id);
  }
</script>

<div class="h-full w-full flex flex-col bg-gray-950">
  <!-- Tab Bar -->
  <div class="flex bg-gray-900 border-b border-gray-800 overflow-x-auto">
    <button
      class="px-4 py-2 text-sm font-medium border-r border-gray-800 hover:bg-gray-800 text-gray-400"
      class:bg-gray-800={!appState.activeTabId}
      class:text-white={!appState.activeTabId}
      onclick={() => (appState.activeTabId = null)}
    >
      Connections
    </button>

    {#each appState.tabs as tab (tab.id)}
      <button
        class="group px-4 py-2 text-sm font-medium border-r border-gray-800 hover:bg-gray-800 flex items-center gap-2 min-w-[120px] max-w-[200px]"
        class:bg-gray-800={appState.activeTabId === tab.id}
        class:text-white={appState.activeTabId === tab.id}
        class:text-gray-400={appState.activeTabId !== tab.id}
        onclick={() => (appState.activeTabId = tab.id)}
      >
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

  <!-- Content Area -->
  <div class="flex-1 overflow-hidden relative">
    {#if !appState.activeTabId}
      <Connection />
    {:else}
      {#each appState.tabs as tab (tab.id)}
        <div
          class="absolute inset-0 bg-gray-950"
          class:hidden={appState.activeTabId !== tab.id}
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
  </div>
</div>
