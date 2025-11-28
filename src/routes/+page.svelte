<script lang="ts">
  import { appState } from "$lib/state.svelte";
  import DataViewer from "../components/DataViewer.svelte";
  import StructureViewer from "../components/StructureViewer.svelte";
  import SqlEditor from "../components/SqlEditor.svelte";
</script>

<div class="h-full w-full">
  {#if appState.currentView === "data"}
    {#if appState.currentTable}
      <DataViewer />
    {:else}
      <div class="flex items-center justify-center h-full text-gray-500">
        Select a table to view data
      </div>
    {/if}
  {:else if appState.currentView === "structure"}
    {#if appState.currentTable}
      <StructureViewer />
    {:else}
      <div class="flex items-center justify-center h-full text-gray-500">
        Select a table to view structure
      </div>
    {/if}
  {:else if appState.currentView === "query"}
    <SqlEditor />
  {/if}
</div>
