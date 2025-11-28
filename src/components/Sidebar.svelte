<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { listTables, listDatabases } from "$lib/db";
    import { Database, Table, Terminal } from "lucide-svelte";

    async function refresh() {
        if (appState.isConnected) {
            appState.databases = await listDatabases();
            appState.tables = await listTables();
        }
    }

    $effect(() => {
        if (appState.isConnected) {
            refresh();
        }
    });
</script>

<div
    class="w-64 bg-gray-900 text-white h-full flex flex-col border-r border-gray-800"
>
    <div class="p-4 border-b border-gray-800 font-bold flex items-center gap-2">
        <Database size={20} />
        <span>Databases</span>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
        {#if appState.databases.length > 0}
            <div class="mb-4">
                <h3
                    class="text-xs uppercase text-gray-500 font-semibold mb-2 px-2"
                >
                    Databases
                </h3>
                <ul>
                    {#each appState.databases as db}
                        <li>
                            <button
                                class="w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm truncate"
                                class:bg-blue-900={appState.currentDatabase ===
                                    db}
                                onclick={() => (appState.currentDatabase = db)}
                            >
                                {db}
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}

        {#if appState.tables.length > 0}
            <div>
                <h3
                    class="text-xs uppercase text-gray-500 font-semibold mb-2 px-2"
                >
                    Tables
                </h3>
                <ul>
                    {#each appState.tables as table}
                        <li>
                            <button
                                class="w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm flex items-center gap-2"
                                class:bg-blue-900={appState.currentTable ===
                                    table}
                                onclick={() => {
                                    appState.currentTable = table;
                                    appState.currentView = "data";
                                }}
                            >
                                <Table size={14} />
                                <span class="truncate">{table}</span>
                            </button>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}
    </div>

    <div class="p-2 border-t border-gray-800">
        <button
            class="w-full flex items-center gap-2 px-2 py-2 rounded hover:bg-gray-800 text-sm"
            onclick={() => (appState.currentView = "query")}
        >
            <Terminal size={16} />
            <span>SQL Editor</span>
        </button>
    </div>
</div>
