<script lang="ts">
    import { untrack } from "svelte";
    import { appState, type Tab } from "$lib/state.svelte";
    import { getTableData, getTableStructure, type QueryResult } from "$lib/db";

    let { tab } = $props<{ tab: Tab }>();

    let loading = $state(false);
    let error = $state("");

    // Initialize defaults if missing
    if (!tab.page) tab.page = 1;
    if (!tab.pageSize) tab.pageSize = 50;
    // Don't initialize tab.data to [] here, so we can distinguish "not loaded" (undefined) from "loaded but empty" ([])
    if (!tab.columns) tab.columns = [];
    if (tab.totalRows === undefined) tab.totalRows = 0;

    let totalPages = $derived(
        Math.ceil((tab.totalRows || 0) / (tab.pageSize || 50)) || 1,
    );

    $effect(() => {
        if (tab.table && (!tab.columns || tab.columns.length === 0)) {
            untrack(() => loadStructure());
        }

        // Load data if empty or if explicit refresh needed (we can add a timestamp later if needed)
        // Only load if data is undefined (never loaded)
        if (tab.data === undefined && !loading && !error) {
            untrack(() => loadData());
        }
    });

    async function loadStructure() {
        if (!tab.table) return;
        try {
            const structure = await getTableStructure(
                tab.connectionId,
                tab.table,
            );
            // The structure returns objects with column_name, data_type, etc.
            // We just need the names for the headers for now.
            // Based on get_table_structure implementation:
            // SELECT column_name, data_type, is_nullable ...
            tab.columns = structure.map((col: any) => col.column_name);
        } catch (e) {
            console.error("Failed to load structure", e);
        }
    }

    function handlePageChange(newPage: number) {
        if (newPage >= 1 && newPage <= totalPages) {
            tab.page = newPage;
            loadData();
        }
    }

    async function loadData() {
        if (!tab.table) return;

        loading = true;
        error = "";
        try {
            const offset = ((tab.page || 1) - 1) * (tab.pageSize || 50);
            const result = await getTableData(
                tab.connectionId,
                tab.table,
                tab.pageSize || 50,
                offset,
            );

            tab.data = result.rows;
            // Only update columns if we don't have them yet, or if the result has them
            // But usually we trust loadStructure for the initial view.
            // However, getTableData returns columns too. Let's ensure they match or just update.
            if (!tab.columns || tab.columns.length === 0) {
                tab.columns = result.columns;
            }
            tab.totalRows = result.total_rows || 0;
        } catch (e: any) {
            error = e.message || "Failed to load data";
        } finally {
            loading = false;
        }
    }

    function formatCell(value: any): string {
        if (value === null || value === undefined) {
            return "";
        }
        if (typeof value === "object") {
            return JSON.stringify(value);
        }
        return String(value);
    }
</script>

<div class="h-full flex flex-col">
    <div
        class="p-4 border-b border-gray-800 flex justify-between items-center bg-gray-900"
    >
        <h2 class="text-lg font-bold flex items-center gap-2">
            <span class="text-gray-400">Table:</span>
            {tab.table}
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
                onclick={() => {
                    tab.type = "structure";
                }}
            >
                Structure
            </button>
        </div>
    </div>

    <div class="flex-1 overflow-auto bg-gray-950 p-4 flex flex-col">
        <div class="flex-1 overflow-auto">
            {#if error}
                <div
                    class="text-red-500 p-4 border border-red-900/50 bg-red-900/20 rounded"
                >
                    Error: {error}
                </div>
            {:else if !tab.columns || tab.columns.length === 0}
                {#if loading}
                    <div
                        class="flex items-center justify-center h-full text-gray-500"
                    >
                        Loading structure...
                    </div>
                {:else}
                    <div
                        class="flex items-center justify-center h-full text-gray-500"
                    >
                        No structure found
                    </div>
                {/if}
            {:else}
                <div
                    class="overflow-x-auto border border-gray-800 rounded h-full flex flex-col relative"
                >
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead
                            class="bg-gray-900 text-gray-400 font-medium sticky top-0 z-10"
                        >
                            <tr>
                                {#each tab.columns || [] as header}
                                    <th
                                        class="px-4 py-2 border-b border-gray-800 bg-gray-900"
                                        >{header}</th
                                    >
                                {/each}
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800">
                            {#if loading && (!tab.data || tab.data.length === 0)}
                                <tr>
                                    <td
                                        colspan={tab.columns.length}
                                        class="p-4 text-center text-gray-500"
                                    >
                                        Loading data...
                                    </td>
                                </tr>
                            {:else if !tab.data || tab.data.length === 0}
                                <tr>
                                    <td
                                        colspan={tab.columns.length}
                                        class="p-4 text-center text-gray-500"
                                    >
                                        No data found
                                    </td>
                                </tr>
                            {:else}
                                {#each tab.data || [] as row}
                                    <tr class="hover:bg-gray-900/50">
                                        {#each row as cell}
                                            <td
                                                class="px-4 py-2 max-w-xs truncate"
                                                title={formatCell(cell)}
                                                >{formatCell(cell)}</td
                                            >
                                        {/each}
                                    </tr>
                                {/each}
                            {/if}
                        </tbody>
                    </table>
                    {#if loading && tab.data && tab.data.length > 0}
                        <div
                            class="absolute inset-0 bg-gray-950/50 flex items-center justify-center z-20"
                        >
                            <span
                                class="text-white bg-gray-800 px-3 py-1 rounded shadow"
                                >Loading...</span
                            >
                        </div>
                    {/if}
                </div>
            {/if}
        </div>

        <!-- Pagination Controls -->
        {#if !error && (tab.columns?.length || 0) > 0}
            <div
                class="flex justify-between items-center pt-4 border-t border-gray-800 mt-4 text-sm text-gray-400"
            >
                <div>
                    Showing {((tab.page || 1) - 1) * (tab.pageSize || 50) + 1} to
                    {Math.min(
                        (tab.page || 1) * (tab.pageSize || 50),
                        tab.totalRows || 0,
                    )} of {tab.totalRows} rows
                </div>
                <div class="flex gap-2 items-center">
                    <button
                        class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={tab.page === 1}
                        onclick={() => handlePageChange((tab.page || 1) - 1)}
                    >
                        Previous
                    </button>
                    <span>Page {tab.page} of {totalPages}</span>
                    <button
                        class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={tab.page === totalPages}
                        onclick={() => handlePageChange((tab.page || 1) + 1)}
                    >
                        Next
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>
