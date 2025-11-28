<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { getTableStructure } from "$lib/db";

    let structure = $state<any[]>([]);
    let loading = $state(false);
    let error = $state("");

    $effect(() => {
        if (appState.currentTable) {
            loadStructure();
        }
    });

    async function loadStructure() {
        loading = true;
        error = "";
        structure = [];
        try {
            structure = await getTableStructure(appState.currentTable);
        } catch (e: any) {
            error = e.message || "Failed to load structure";
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
            <span class="text-gray-400">Structure:</span>
            {appState.currentTable}
        </h2>
        <div class="flex gap-2">
            <button
                class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-sm"
                onclick={() => (appState.currentView = "data")}
            >
                View Data
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
        {:else if structure.length === 0}
            <div class="flex items-center justify-center h-full text-gray-500">
                No structure found
            </div>
        {:else}
            <div class="overflow-x-auto border border-gray-800 rounded">
                <table class="w-full text-left text-sm whitespace-nowrap">
                    <thead class="bg-gray-900 text-gray-400 font-medium">
                        <tr>
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Column Name</th
                            >
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Data Type</th
                            >
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Nullable</th
                            >
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-800">
                        {#each structure as col}
                            <tr class="hover:bg-gray-900/50">
                                <td class="px-4 py-2 font-mono text-blue-400"
                                    >{col.column_name}</td
                                >
                                <td class="px-4 py-2 text-yellow-500"
                                    >{col.data_type}</td
                                >
                                <td class="px-4 py-2">
                                    <span
                                        class:text-green-500={col.is_nullable ===
                                            "YES"}
                                        class:text-red-500={col.is_nullable ===
                                            "NO"}
                                    >
                                        {col.is_nullable}
                                    </span>
                                </td>
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>
        {/if}
    </div>
</div>
