<script lang="ts">
    import { untrack } from "svelte";
    import { appState } from "$lib/state.svelte";
    import {
        getTableStructure,
        getTableIndexes,
        type ColumnDefinition,
        type IndexDefinition,
    } from "$lib/db";

    let { connectionId, tableName } = $props<{
        connectionId: string;
        tableName: string;
    }>();

    let structure = $state<ColumnDefinition[]>([]);
    let indexes = $state<IndexDefinition[]>([]);
    let loading = $state(false);
    let error = $state("");

    $effect(() => {
        if (connectionId && tableName) {
            untrack(() => loadStructure());
        }
    });

    async function loadStructure() {
        loading = true;
        error = "";
        structure = [];
        indexes = [];
        try {
            const [struct, idxs] = await Promise.all([
                getTableStructure(connectionId, tableName),
                getTableIndexes(connectionId, tableName),
            ]);
            structure = struct;
            indexes = idxs;
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
            {tableName}
        </h2>
        <div class="flex gap-2">
            <button
                class="px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-sm"
                onclick={() => {
                    const tab = appState.tabs.find(
                        (t) => t.id === appState.activeTabId,
                    );
                    if (tab) {
                        tab.type = "data";
                    }
                }}
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
            <div class="overflow-x-auto border border-gray-800 rounded mb-4">
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
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Default</th
                            >
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Foreign Key</th
                            >
                            <th class="px-4 py-2 border-b border-gray-800"
                                >Comment</th
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
                                <td class="px-4 py-2 text-gray-400"
                                    >{col.column_default || "-"}</td
                                >
                                <td class="px-4 py-2 text-purple-400"
                                    >{col.foreign_key || "-"}</td
                                >
                                <td class="px-4 py-2 text-gray-500 italic"
                                    >{col.comment || "-"}</td
                                >
                            </tr>
                        {/each}
                    </tbody>
                </table>
            </div>

            {#if indexes.length > 0}
                <div class="mb-2">
                    <h3 class="text-md font-bold text-gray-400 mb-2">
                        Indexes
                    </h3>
                    <div class="overflow-x-auto border border-gray-800 rounded">
                        <table
                            class="w-full text-left text-sm whitespace-nowrap"
                        >
                            <thead
                                class="bg-gray-900 text-gray-400 font-medium"
                            >
                                <tr>
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Index Name</th
                                    >
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Algorithm</th
                                    >
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Unique</th
                                    >
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Columns</th
                                    >
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Condition</th
                                    >
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >Comment</th
                                    >
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-800">
                                {#each indexes as idx}
                                    <tr class="hover:bg-gray-900/50">
                                        <td
                                            class="px-4 py-2 font-mono text-green-400"
                                        >
                                            {idx.index_name}
                                            {#if idx.is_primary}
                                                <span
                                                    class="ml-2 text-xs bg-blue-900 text-blue-300 px-1 rounded"
                                                    >PK</span
                                                >
                                            {/if}
                                        </td>
                                        <td class="px-4 py-2 text-gray-400"
                                            >{idx.index_algorithm}</td
                                        >
                                        <td class="px-4 py-2">
                                            <span
                                                class:text-green-500={idx.is_unique}
                                                class:text-gray-600={!idx.is_unique}
                                            >
                                                {idx.is_unique ? "YES" : "NO"}
                                            </span>
                                        </td>
                                        <td class="px-4 py-2 text-yellow-500"
                                            >{idx.column_names}</td
                                        >
                                        <td class="px-4 py-2 text-gray-400"
                                            >{idx.condition || "-"}</td
                                        >
                                        <td
                                            class="px-4 py-2 text-gray-500 italic"
                                            >{idx.comment || "-"}</td
                                        >
                                    </tr>
                                {/each}
                            </tbody>
                        </table>
                    </div>
                </div>
            {/if}
        {/if}
    </div>
</div>
