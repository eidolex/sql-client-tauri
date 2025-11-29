<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { disconnectDb } from "$lib/db";
    import { Plus, LogOut } from "lucide-svelte";

    function getInitials(name: string) {
        return name
            .split(" ")
            .map((n) => n[0])
            .join("")
            .substring(0, 2)
            .toUpperCase();
    }

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
</script>

<div
    class="w-[70px] bg-gray-900 border-r border-gray-800 flex flex-col items-center py-4 gap-4 shrink-0"
>
    <!-- New Connection / Home -->
    <button
        class="w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-200 group relative"
        class:bg-blue-600={!appState.selectedConnectionId}
        class:bg-gray-800={appState.selectedConnectionId}
        class:hover:bg-blue-600={appState.selectedConnectionId}
        onclick={() => (appState.selectedConnectionId = null)}
        title="New Connection"
    >
        <Plus size={24} class="text-white" />
    </button>

    <div class="w-8 h-[1px] bg-gray-800"></div>

    <!-- Active Connections -->
    <div
        class="flex-1 w-full flex flex-col items-center gap-3 overflow-y-auto no-scrollbar"
    >
        {#each appState.activeConnections as connection (connection.id)}
            <button
                class="w-12 h-12 rounded-xl flex items-center justify-center transition-all duration-200 relative group"
                class:bg-gray-800={appState.selectedConnectionId !==
                    connection.id}
                class:bg-blue-600={appState.selectedConnectionId ===
                    connection.id}
                class:text-white={true}
                onclick={() => (appState.selectedConnectionId = connection.id)}
                oncontextmenu={(e) => handleContextMenu(e, connection.id)}
                title={connection.config.name}
            >
                <span class="font-bold text-sm">
                    {getInitials(connection.config.name)}
                </span>

                <!-- Active Indicator -->
                {#if appState.selectedConnectionId === connection.id}
                    <div
                        class="absolute -left-4 top-1/2 -translate-y-1/2 w-1 h-8 bg-white rounded-r-full"
                    ></div>
                {/if}
            </button>
        {/each}
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
                <LogOut size={14} />
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
