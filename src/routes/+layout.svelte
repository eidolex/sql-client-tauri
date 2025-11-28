<script>
    import "../app.css";
    import { appState } from "$lib/state.svelte";
    import Sidebar from "../components/Sidebar.svelte";
    import Connection from "../components/Connection.svelte";

    let { children } = $props();
</script>

<div
    class="h-screen w-screen bg-gray-950 text-white overflow-hidden flex font-sans antialiased"
>
    {#if !appState.isConnected}
        <div class="w-full h-full">
            <Connection />
        </div>
    {:else}
        <Sidebar />
        <main class="flex-1 h-full overflow-hidden bg-gray-900 relative">
            {@render children()}
        </main>
    {/if}
</div>
