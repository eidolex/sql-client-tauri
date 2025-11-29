<script>
    import "../app.css";
    import Sidebar from "../components/Sidebar.svelte";
    import ActivityBar from "../components/ActivityBar.svelte";
    import { appState } from "$lib/state.svelte";
    import { ModeWatcher } from "mode-watcher";

    let { children } = $props();
</script>

<ModeWatcher />

<div
    class="h-screen w-screen bg-background text-foreground overflow-hidden flex font-sans antialiased"
>
    <ActivityBar />
    {#if appState.selectedConnectionId}
        <Sidebar />
    {/if}
    <main class="flex-1 h-full overflow-hidden bg-background relative">
        {@render children()}
    </main>
</div>
