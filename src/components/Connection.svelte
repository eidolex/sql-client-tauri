<script lang="ts">
    import { connectDb } from "$lib/db";
    import { appState } from "$lib/state.svelte";

    let connectionString = $state(
        "postgres://postgres:password@localhost:5432/postgres",
    );
    let error = $state("");
    let loading = $state(false);

    async function connect() {
        loading = true;
        error = "";
        try {
            await connectDb(connectionString);
            appState.isConnected = true;
        } catch (e: any) {
            error = e.message || "Failed to connect";
        } finally {
            loading = false;
        }
    }
</script>

<div class="flex items-center justify-center h-full bg-gray-950 text-white">
    <div
        class="w-full max-w-md p-8 bg-gray-900 rounded-lg shadow-xl border border-gray-800"
    >
        <h2 class="text-2xl font-bold mb-6 text-center">Connect to Database</h2>

        <div class="space-y-4">
            <div>
                <label
                    for="conn-string"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >Connection String</label
                >
                <input
                    id="conn-string"
                    type="text"
                    bind:value={connectionString}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 text-white focus:outline-none focus:border-blue-500"
                    placeholder="postgres://user:pass@host:port/db"
                />
            </div>

            {#if error}
                <div
                    class="text-red-500 text-sm p-2 bg-red-900/20 rounded border border-red-900/50"
                >
                    {error}
                </div>
            {/if}

            <button
                onclick={connect}
                disabled={loading}
                class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-4 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
                {loading ? "Connecting..." : "Connect"}
            </button>
        </div>
    </div>
</div>
