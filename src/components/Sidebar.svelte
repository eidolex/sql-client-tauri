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
    } from "lucide-svelte";

    let expandedConnections = $state<Set<string>>(new Set());
    let contextMenu = $state<{
        visible: boolean;
        x: number;
        y: number;
        connectionId: string | null;
    }>({ visible: false, x: 0, y: 0, connectionId: null });

    function handleContextMenu(e: MouseEvent, connectionId: string) {
        e.preventDefault();
        contextMenu = {
            visible: true,
            x: e.clientX,
            y: e.clientY,
            connectionId,
        };
    }

    function closeContextMenu() {
        contextMenu.visible = false;
    }

    async function closeConnection() {
        if (contextMenu.connectionId) {
            try {
                await disconnectDb(contextMenu.connectionId);
                appState.removeConnection(contextMenu.connectionId);
            } catch (e) {
                console.error("Failed to disconnect", e);
            }
        }
        closeContextMenu();
    }

    function toggleConnection(id: string) {
        if (expandedConnections.has(id)) {
            expandedConnections.delete(id);
        } else {
            expandedConnections.add(id);
        }
        expandedConnections = new Set(expandedConnections);
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

    async function openDatabaseConnection(
        connection: ActiveConnection,
        database: string,
    ) {
        // Check if we already have a connection to this database
        const existing = appState.activeConnections.find(
            (c) =>
                c.config.host === connection.config.host &&
                c.config.port === connection.config.port &&
                c.currentDatabase === database,
        );

        if (existing) {
            // Just expand it if it exists (logic to expand can be added if needed, for now just alert or ignore)
            if (!expandedConnections.has(existing.id)) {
                toggleConnection(existing.id);
            }
            return;
        }

        try {
            // Connect to new DB as a separate connection
            const newConfig = { ...connection.config, database };
            // Generate a temporary name for the new connection view
            newConfig.name = `${connection.config.name} (${database})`;

            const newId = await connectDb(newConfig);
            const tables = await listTables(newId);

            appState.addConnection({
                id: newId,
                config: newConfig,
                databases: connection.databases, // Reuse the list of databases
                tables,
                currentDatabase: database,
            });

            // Auto expand the new connection
            expandedConnections.add(newId);
            expandedConnections = new Set(expandedConnections);
        } catch (e) {
            console.error("Failed to open database connection", e);
            alert("Failed to open database connection: " + e);
        }
    }
</script>

<div
    class="w-64 bg-gray-900 text-white h-full flex flex-col border-r border-gray-800"
>
    <div class="p-4 border-b border-gray-800 font-bold flex items-center gap-2">
        <Database size={20} />
        <span>Explorer</span>
    </div>

    <div class="flex-1 overflow-y-auto p-2">
        {#each appState.activeConnections as connection (connection.id)}
            <div class="mb-2">
                <button
                    class="w-full flex items-center gap-2 px-2 py-1 hover:bg-gray-800 rounded text-sm font-medium"
                    onclick={() => toggleConnection(connection.id)}
                    oncontextmenu={(e) => handleContextMenu(e, connection.id)}
                >
                    {#if expandedConnections.has(connection.id)}
                        <ChevronDown size={14} />
                    {:else}
                        <ChevronRight size={14} />
                    {/if}
                    <Plug size={14} class="text-blue-400" />
                    <span class="truncate">{connection.config.name}</span>
                </button>

                {#if expandedConnections.has(connection.id)}
                    <div class="ml-4 mt-1 border-l border-gray-800 pl-2">
                        <div
                            class="text-xs text-gray-500 uppercase font-semibold mb-1 px-2"
                        >
                            Databases
                        </div>
                        {#each connection.databases as db}
                            <div class="mb-1">
                                <div
                                    class="w-full flex items-center gap-2 px-2 py-1 rounded text-sm"
                                    class:text-blue-400={connection.currentDatabase ===
                                        db}
                                    class:text-gray-400={connection.currentDatabase !==
                                        db}
                                    onclick={() =>
                                        openDatabaseConnection(connection, db)}
                                    role="button"
                                    tabindex="0"
                                    onkeydown={(e) =>
                                        e.key === "Enter" &&
                                        openDatabaseConnection(connection, db)}
                                    class:cursor-pointer={connection.currentDatabase !==
                                        db}
                                    class:hover:bg-gray-800={connection.currentDatabase !==
                                        db}
                                >
                                    <Database size={12} />
                                    <span class="truncate">{db}</span>
                                    {#if connection.currentDatabase === db}
                                        <span
                                            class="text-xs text-green-500 ml-auto"
                                            >Active</span
                                        >
                                    {/if}
                                </div>

                                {#if connection.currentDatabase === db}
                                    <div class="ml-4 mt-1">
                                        {#each connection.tables as table}
                                            <button
                                                class="w-full text-left px-2 py-1 rounded hover:bg-gray-800 text-sm flex items-center gap-2"
                                                onclick={() =>
                                                    openTable(
                                                        connection,
                                                        table,
                                                    )}
                                            >
                                                <Table
                                                    size={14}
                                                    class="text-gray-400"
                                                />
                                                <span class="truncate"
                                                    >{table}</span
                                                >
                                            </button>
                                        {/each}
                                    </div>
                                {/if}
                            </div>
                        {/each}

                        <button
                            class="w-full flex items-center gap-2 px-2 py-1 mt-2 hover:bg-gray-800 rounded text-sm text-gray-400"
                            onclick={() => openSqlEditor(connection)}
                        >
                            <Terminal size={14} />
                            <span>SQL Editor</span>
                        </button>
                    </div>
                {/if}
            </div>
        {/each}

        {#if appState.activeConnections.length === 0}
            <div class="text-center text-gray-500 mt-10 text-sm">
                No active connections.
                <br />
                Create a new connection to start.
            </div>
        {/if}
    </div>

    <!-- Context Menu -->
    {#if contextMenu.visible}
        <div
            class="fixed z-50 bg-gray-800 border border-gray-700 rounded shadow-lg py-1 min-w-[150px]"
            style="top: {contextMenu.y}px; left: {contextMenu.x}px;"
        >
            <button
                class="w-full text-left px-4 py-2 text-sm hover:bg-gray-700 text-red-400 flex items-center gap-2"
                onclick={closeConnection}
            >
                <Plug size={14} class="rotate-45" />
                Close Connection
            </button>
        </div>
        <!-- Backdrop to close menu -->
        <div
            class="fixed inset-0 z-40 bg-transparent"
            onclick={closeContextMenu}
            role="button"
            tabindex="-1"
            onkeydown={(e) => e.key === "Escape" && closeContextMenu()}
        ></div>
    {/if}
</div>
