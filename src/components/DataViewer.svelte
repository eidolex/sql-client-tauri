<script lang="ts">
    import { untrack } from "svelte";
    import { appState, type Tab } from "$lib/state.svelte";
    import { getTableData, type QueryResult } from "$lib/db";

    let { tab } = $props<{ tab: Tab }>();

    let loading = $state(false);
    let error = $state("");

    // Initialize defaults if missing
    if (!tab.page) tab.page = 1;
    if (!tab.pageSize) tab.pageSize = 50;
    if (!tab.data) tab.data = [];
    if (!tab.columns) tab.columns = [];
    if (tab.totalRows === undefined) tab.totalRows = 0;

    let totalPages = $derived(
        Math.ceil((tab.totalRows || 0) / (tab.pageSize || 50)) || 1,
    );

    $effect(() => {
        // Load data if empty or if explicit refresh needed (we can add a timestamp later if needed)
        // For now, just load if empty.
        if (tab.data && tab.data.length === 0 && !loading && !error) {
            untrack(() => loadData());
        }
    });

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
            tab.columns = result.columns;
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
            {#if loading}
                <div
                    class="flex items-center justify-center h-full text-gray-500"
                >
                    Loading...
                </div>
            {:else if error}
                <div
                    class="text-red-500 p-4 border border-red-900/50 bg-red-900/20 rounded"
                >
                    Error: {error}
                </div>
            {:else if !tab.data || tab.data.length === 0}
                <div
                    class="flex items-center justify-center h-full text-gray-500"
                >
                    No data found
                </div>
            {:else}
                <div class="overflow-x-auto border border-gray-800 rounded">
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead class="bg-gray-900 text-gray-400 font-medium">
                            <tr>
                                {#each tab.columns || [] as header}
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >{header}</th
                                    >
                                {/each}
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800">
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
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>

        <!-- Pagination Controls -->
        {#if !loading && !error && (tab.data?.length || 0) > 0}
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
