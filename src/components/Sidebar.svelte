<script lang="ts">
    import { appState, type ActiveConnection } from "$lib/state.svelte";
    import { connectDb, disconnectDb, listTables } from "$lib/db";
    import {
        Database,
        Table,
        Terminal,
        ChevronRight,
        ChevronDown,
        Plug,
        LogOut,
    } from "lucide-svelte";

    let expandedDatabases = $state<Set<string>>(new Set());

    // When the selected connection changes, we might want to reset or restore expanded state
    // For now, let's just keep it simple.

    let currentConnection = $derived(
        appState.selectedConnectionId
            ? appState.getConnection(appState.selectedConnectionId)
            : null,
    );

    function toggleDatabase(dbName: string) {
        if (expandedDatabases.has(dbName)) {
            expandedDatabases.delete(dbName);
        } else {
            expandedDatabases.add(dbName);
        }
        expandedDatabases = new Set(expandedDatabases);
    }

    function openTable(connection: ActiveConnection, table: string) {
        // Check if tab already exists
        const existingTab = appState.tabs.find(
            (t) =>
                t.connectionId === connection.id &&
                t.table === table &&
                t.type === "data",
        );

        if (existingTab) {
            appState.activeTabId = existingTab.id;
        } else {
            appState.addTab({
                id: crypto.randomUUID(),
                title: table,
                type: "data",
                connectionId: connection.id,
                database: connection.currentDatabase,
                table: table,
            });
        }
    }

    function openSqlEditor(connection: ActiveConnection) {
        appState.addTab({
            id: crypto.randomUUID(),
            title: `Query: ${connection.config.name}`,
            type: "query",
            connectionId: connection.id,
            database: connection.currentDatabase,
        });
    }

    let connectingDatabases = $state<Set<string>>(new Set());

    async function switchDatabase(
        connection: ActiveConnection,
        database: string,
    ) {
        // If we are already on this database, just toggle expansion
        if (connection.currentDatabase === database) {
            // It's already the active database for this connection
            return;
        }

        // Check if we are already connecting to this database
        const connectingKey = `${connection.id}-${database}`;
        if (connectingDatabases.has(connectingKey)) {
            return;
        }

        const existing = appState.activeConnections.find(
            (c) =>
                c.config.host === connection.config.host &&
                c.config.port == connection.config.port &&
                c.config.username === connection.config.username &&
                c.currentDatabase === database,
        );

        if (existing) {
            appState.selectedConnectionId = existing.id;
            return;
        }

        try {
            connectingDatabases.add(connectingKey);
            connectingDatabases = new Set(connectingDatabases);

            // Connect to new DB as a separate connection
            const newConfig = { ...connection.config, database };
            // Generate a temporary name for the new connection view
            newConfig.name = `${connection.config.name} (${database})`;

            const newId = await connectDb(newConfig);
            const tables = await listTables(newId);

            const newConnection = {
                id: newId,
                config: newConfig,
                databases: connection.databases, // Reuse the list of databases
                tables,
                currentDatabase: database,
            };

            appState.addConnection(newConnection);
            // addConnection already sets selectedConnectionId
        } catch (e) {
            console.error("Failed to open database connection", e);
            alert("Failed to open database connection: " + e);
        } finally {
            connectingDatabases.delete(connectingKey);
            connectingDatabases = new Set(connectingDatabases);
        }
    }

    async function closeCurrentConnection() {
        if (currentConnection) {
            try {
                await disconnectDb(currentConnection.id);
                appState.removeConnection(currentConnection.id);
            } catch (e) {
                console.error("Failed to disconnect", e);
            }
        }
    }
</script>

<div
    class="w-64 bg-gray-900 text-white h-full flex flex-col border-r border-gray-800"
>
    {#if !currentConnection}
        <!-- Home / No Connection Selected -->
        <div
            class="p-4 border-b border-gray-800 font-bold flex items-center gap-2"
        >
            <Database size={20} />
            <span>Explorer</span>
        </div>
        <div class="p-4 text-sm text-gray-400">
            Select a connection from the left bar or create a new one.
        </div>
    {:else}
        <!-- Active Connection Context -->
        <div
            class="p-4 border-b border-gray-800 flex items-center justify-between group"
        >
            <div
                class="font-bold truncate"
                title={currentConnection.config.name}
            >
                {currentConnection.config.name}
            </div>
            <button
                class="text-gray-500 hover:text-red-400 opacity-0 group-hover:opacity-100 transition-opacity"
                onclick={closeCurrentConnection}
                title="Close Connection"
            >
                <LogOut size={16} />
            </button>
        </div>

        <div class="flex-1 overflow-y-auto p-2">
            <div
                class="text-xs text-gray-500 uppercase font-semibold mb-2 px-2"
            >
                Databases
            </div>

            {#each currentConnection.databases as db}
                <div class="mb-1">
                    <button
                        class="w-full flex items-center gap-2 px-2 py-1 rounded text-sm hover:bg-gray-800 text-left"
                        class:text-blue-400={currentConnection.currentDatabase ===
                            db}
                        class:font-medium={currentConnection.currentDatabase ===
                            db}
                        class:text-gray-400={currentConnection.currentDatabase !==
                            db}
                        onclick={() => switchDatabase(currentConnection!, db)}
                        disabled={connectingDatabases.has(
                            `${currentConnection.id}-${db}`,
                        )}
                    >
                        {#if connectingDatabases.has(`${currentConnection.id}-${db}`)}
                            <div
                                class="w-3.5 h-3.5 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"
                            ></div>
                        {:else}
                            <Database size={14} />
                        {/if}
                        <span class="truncate flex-1">{db}</span>
                        {#if currentConnection.currentDatabase === db}
                            <div class="w-2 h-2 rounded-full bg-blue-500"></div>
                        {/if}
                    </button>

                    <!-- Show tables if this is the active database -->
                    {#if currentConnection.currentDatabase === db}
                        <div class="ml-4 mt-1 border-l border-gray-800 pl-2">
                            <button
                                class="w-full flex items-center gap-2 px-2 py-1 mb-1 hover:bg-gray-800 rounded text-sm text-gray-400"
                                onclick={() =>
                                    openSqlEditor(currentConnection!)}
                            >
                                <Terminal size={14} />
                                <span>SQL Editor</span>
                            </button>

                            {#each currentConnection.tables as table}
                                <button
                                    class="w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm flex items-center gap-2 text-gray-300"
                                    onclick={() =>
                                        openTable(currentConnection!, table)}
                                >
                                    <Table size={14} class="text-gray-500" />
                                    <span class="truncate">{table}</span>
                                </button>
                            {/each}
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>
