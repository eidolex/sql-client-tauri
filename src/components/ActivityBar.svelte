<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { Plus } from "lucide-svelte";

    function getInitials(name: string) {
        return name
            .split(" ")
            .map((n) => n[0])
            .join("")
            .substring(0, 2)
            .toUpperCase();
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
</div>
