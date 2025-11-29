<script lang="ts">
    import { untrack } from "svelte";
    import { appState, type Tab } from "$lib/state.svelte";
    import { getTableData, getTableStructure, type QueryResult } from "$lib/db";

    let { tab } = $props<{ tab: Tab }>();

    let loading = $state(false);
    let error = $state("");
    let showFilters = $state(false);

    interface Filter {
        field: string;
        operator: string;
        value: string;
    }

    interface Sort {
        field: string;
        order: "ASC" | "DESC";
    }

    let filters = $state<Filter[]>([]);
    let sorts = $state<Sort[]>([]);

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
                filters,
                sorts,
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

    function addFilter() {
        if (tab.columns && tab.columns.length > 0) {
            filters.push({
                field: tab.columns[0],
                operator: "=",
                value: "",
            });
        }
    }

    function removeFilter(index: number) {
        filters.splice(index, 1);
    }

    function applyFilters() {
        tab.page = 1; // Reset to first page when filtering
        loadData();
    }

    function handleHeaderClick(field: string, event: MouseEvent) {
        const existingSortIndex = sorts.findIndex((s) => s.field === field);
        const isShift = event.shiftKey;

        if (isShift) {
            if (existingSortIndex !== -1) {
                // Toggle existing
                if (sorts[existingSortIndex].order === "ASC") {
                    sorts[existingSortIndex].order = "DESC";
                } else {
                    sorts.splice(existingSortIndex, 1);
                }
            } else {
                // Add new
                sorts.push({ field, order: "ASC" });
            }
        } else {
            if (existingSortIndex !== -1 && sorts.length === 1) {
                // Toggle existing single sort
                if (sorts[0].order === "ASC") {
                    sorts[0].order = "DESC";
                } else {
                    sorts = [];
                }
            } else {
                // Replace with new single sort
                sorts = [{ field, order: "ASC" }];
            }
        }
        loadData();
    }

    function getSortIndicator(field: string) {
        const index = sorts.findIndex((s) => s.field === field);
        if (index === -1) return null;
        return {
            order: sorts[index].order,
            index: sorts.length > 1 ? index + 1 : null,
        };
    }
</script>

<div class="h-full flex flex-col">
    <div class="p-4 border-b border-gray-800 flex flex-col gap-4 bg-gray-900">
        <div class="flex justify-between items-center">
            <h2 class="text-lg font-bold flex items-center gap-2">
                <span class="text-gray-400">Table:</span>
                {tab.table}
            </h2>
            <div class="flex gap-2">
                <button
                    class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-sm flex items-center gap-2"
                    onclick={() => (showFilters = !showFilters)}
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-4 w-4"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z"
                        />
                    </svg>
                    Filter
                    {#if filters.length > 0}
                        <span
                            class="bg-blue-600 text-white text-xs rounded-full px-1.5"
                            >{filters.length}</span
                        >
                    {/if}
                </button>
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

        {#if showFilters}
            <div class="bg-gray-950 p-4 rounded border border-gray-800">
                <div class="flex flex-col gap-2">
                    {#each filters as filter, i}
                        <div class="flex gap-2 items-center">
                            <select
                                class="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm"
                                bind:value={filter.field}
                            >
                                {#each tab.columns || [] as col}
                                    <option value={col}>{col}</option>
                                {/each}
                            </select>
                            <select
                                class="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm"
                                bind:value={filter.operator}
                            >
                                <option value="=">=</option>
                                <option value=">=">&gt;=</option>
                                <option value="<=">&lt;=</option>
                                <option value=">">&gt;</option>
                                <option value="<">&lt;</option>
                                <option value="contain">contain</option>
                                <option value="start with">start with</option>
                                <option value="end with">end with</option>
                                <option value="not null">not null</option>
                                <option value="is null">is null</option>
                            </select>
                            {#if filter.operator !== "not null" && filter.operator !== "is null"}
                                <input
                                    type="text"
                                    class="bg-gray-800 border border-gray-700 rounded px-2 py-1 text-sm flex-1"
                                    placeholder="Value"
                                    bind:value={filter.value}
                                />
                            {/if}
                            <button
                                class="text-red-400 hover:text-red-300 p-1"
                                onclick={() => removeFilter(i)}
                            >
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    class="h-4 w-4"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke="currentColor"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M6 18L18 6M6 6l12 12"
                                    />
                                </svg>
                            </button>
                        </div>
                    {/each}
                    <div class="flex gap-2 mt-2">
                        <button
                            class="text-sm text-blue-400 hover:text-blue-300 flex items-center gap-1"
                            onclick={addFilter}
                        >
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                class="h-4 w-4"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M12 4v16m8-8H4"
                                />
                            </svg>
                            Add Filter
                        </button>
                        <div class="flex-1"></div>
                        <button
                            class="px-3 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm"
                            onclick={applyFilters}
                        >
                            Apply
                        </button>
                    </div>
                </div>
            </div>
        {/if}
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
                                    {@const sort = getSortIndicator(header)}
                                    <th
                                        class="px-4 py-2 border-b border-gray-800 bg-gray-900 cursor-pointer hover:bg-gray-800 select-none"
                                        onclick={(e) =>
                                            handleHeaderClick(header, e)}
                                    >
                                        <div class="flex items-center gap-1">
                                            {header}
                                            {#if sort}
                                                <span
                                                    class="text-xs text-blue-400 flex items-center"
                                                >
                                                    {#if sort.order === "ASC"}
                                                        ▲
                                                    {:else}
                                                        ▼
                                                    {/if}
                                                    {#if sort.index}
                                                        <span
                                                            class="ml-0.5 text-[10px]"
                                                            >{sort.index}</span
                                                        >
                                                    {/if}
                                                </span>
                                            {/if}
                                        </div>
                                    </th>
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
