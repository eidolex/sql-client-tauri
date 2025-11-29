<script>
    import "../app.css";
    import Sidebar from "../components/Sidebar.svelte";
    import ActivityBar from "../components/ActivityBar.svelte";
    import { appState } from "$lib/state.svelte";

    let { children } = $props();
</script>

<div
    class="h-screen w-screen bg-gray-950 text-white overflow-hidden flex font-sans antialiased"
>
    <ActivityBar />
    {#if appState.selectedConnectionId}
        <Sidebar />
    {/if}
    <main class="flex-1 h-full overflow-hidden bg-gray-900 relative">
        {@render children()}
    </main>
</div>
