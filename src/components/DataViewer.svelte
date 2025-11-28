<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { getTableData, type QueryResult } from "$lib/db";

    let result = $state<QueryResult>({ columns: [], rows: [] });
    let loading = $state(false);
    let error = $state("");

    $effect(() => {
        if (appState.currentTable) {
            loadData();
        }
    });

    async function loadData() {
        loading = true;
        error = "";
        result = { columns: [], rows: [] };
        try {
            result = await getTableData(appState.currentTable);
        } catch (e: any) {
            error = e.message || "Failed to load data";
        } finally {
            loading = false;
        }
    }
</script>

<div class="h-full flex flex-col">
    <div
        class="p-4 border-b border-gray-800 flex justify-between items-center bg-gray-900"
    >
        <h2 class="text-lg font-bold flex items-center gap-2">
            <span class="text-gray-400">Table:</span>
            {appState.currentTable}
        </h2>
        <div class="flex gap-2">
            <button
                class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-sm"
                onclick={loadData}
            >
                Refresh
            </button>
            <button
                class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-sm"
                onclick={() => (appState.currentView = "structure")}
            >
                Structure
            </button>
        </div>
    </div>

    <div class="flex-1 overflow-auto bg-gray-950 p-4">
        {#if loading}
            <div class="flex items-center justify-center h-full text-gray-500">
                Loading...
            </div>
        {:else if error}
            <div
                class="text-red-500 p-4 border border-red-900/50 bg-red-900/20 rounded"
            >
                Error: {error}
            </div>
        {:else if result.rows.length === 0}
            <div class="flex items-center justify-center h-full text-gray-500">
                No data found
            </div>
        {:else}
            <div class="overflow-x-auto border border-gray-800 rounded">
                <table class="w-full text-left text-sm whitespace-nowrap">
                    <thead class="bg-gray-900 text-gray-400 font-medium">
                        <tr>
                            {#each result.columns as header}
                                <th class="px-4 py-2 border-b border-gray-800"
                                    >{header}</th
                                >
                            {/each}
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800">
                        {#each result.rows as row}
                            <tr class="hover:bg-gray-900/50">
                                {#each row as cell}
                                    <td
                                        class="px-4 py-2 max-w-xs truncate"
                                        title={String(cell)}>{String(cell)}</td
                                    >
                                {/each}
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>
</div>
