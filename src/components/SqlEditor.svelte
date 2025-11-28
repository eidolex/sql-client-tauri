<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { EditorView, basicSetup } from "codemirror";
    import { sql } from "@codemirror/lang-sql";
    import { getDatabaseSchema, executeQuery, type QueryResult } from "$lib/db";
    import { appState } from "$lib/state.svelte";

    let { connectionId, initialQuery } = $props<{
        connectionId: string;
        initialQuery?: string;
    }>();

    let element: HTMLElement;
    let view: EditorView;
    let query = $state(
        initialQuery || "SELECT * FROM information_schema.tables;",
    );
    let result = $state<QueryResult>({ columns: [], rows: [] });
    let error = $state("");
    let loading = $state(false);

    onMount(async () => {
        let schema = {};
        try {
            schema = await getDatabaseSchema(connectionId);
        } catch (e) {
            console.error("Failed to load schema for autocomplete", e);
        }

        view = new EditorView({
            doc: query,
            extensions: [
                basicSetup,
                sql({ schema }),
                EditorView.theme(
                    {
                        "&": { height: "100%", fontSize: "14px" },
                        ".cm-scroller": { overflow: "auto" },
                        ".cm-content": { fontFamily: "monospace" },
                    },
                    { dark: true },
                ),
            ],
            parent: element,
            dispatch: (tr) => {
                view.update([tr]);
                if (tr.docChanged) {
                    query = view.state.doc.toString();
                    const tab = appState.tabs.find(
                        (t) => t.id === appState.activeTabId,
                    );
                    if (tab && tab.type === "query") {
                        tab.query = query;
                    }
                }
            },
        });
    });

    onDestroy(() => {
        view?.destroy();
    });

    async function runQuery() {
        loading = true;
        error = "";
        result = { columns: [], rows: [] };

        let queryToRun = query;
        if (view) {
            const selection = view.state.selection.main;
            if (!selection.empty) {
                queryToRun = view.state.sliceDoc(selection.from, selection.to);
            }
        }

        try {
            result = await executeQuery(connectionId, queryToRun);
        } catch (e: any) {
            error = e.message || "Query failed";
        } finally {
            loading = false;
        }
    }
</script>

<div class="h-full flex flex-col">
    <div class="h-1/2 flex flex-col border-b border-gray-800">
        <div
            class="p-2 bg-gray-900 border-b border-gray-800 flex justify-between items-center"
        >
            <span class="font-bold text-sm text-gray-400">SQL Editor</span>
            <button
                class="px-3 py-1 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm font-bold flex items-center gap-2"
                onclick={runQuery}
                disabled={loading}
            >
                {#if loading}
                    <span>Running...</span>
                {:else}
                    <span>Run</span>
                {/if}
            </button>
        </div>
        <div
            class="flex-1 overflow-hidden bg-[#282c34]"
            bind:this={element}
        ></div>
    </div>

    <div class="h-1/2 flex flex-col bg-gray-950">
        <div
            class="p-2 bg-gray-900 border-b border-gray-800 font-bold text-sm text-gray-400"
        >
            Results
        </div>
        <div class="flex-1 overflow-auto p-4">
            {#if error}
                <div
                    class="text-red-500 p-4 border border-red-900/50 bg-red-900/20 rounded"
                >
                    Error: {error}
                </div>
            {:else if result.rows.length === 0 && !loading}
                <div class="text-gray-500">No results</div>
            {:else if result.rows.length > 0}
                <div class="overflow-x-auto border border-gray-800 rounded">
                    <table class="w-full text-left text-sm whitespace-nowrap">
                        <thead class="bg-gray-900 text-gray-400 font-medium">
                            <tr>
                                {#each result.columns as header}
                                    <th
                                        class="px-4 py-2 border-b border-gray-800"
                                        >{header}</th
                                    >
                                {/each}
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-800">
                            {#each result.rows as row}
                                <tr class="hover:bg-gray-900/50">
                                    {#each row as cell}
                                        <td
                                            class="px-4 py-2 max-w-xs truncate"
                                            title={String(cell)}
                                            >{String(cell)}</td
                                        >
                                    {/each}
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                </div>
            {/if}
        </div>
    </div>
</div>
