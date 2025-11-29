<script lang="ts">
    import { untrack } from "svelte";
    import { appState } from "$lib/state.svelte";
    import {
        getTableStructure,
        getTableIndexes,
        type ColumnDefinition,
        type IndexDefinition,
    } from "$lib/db";
    import { Button } from "$lib/components/ui/button";
    import * as Table from "$lib/components/ui/table";
    import { Badge } from "$lib/components/ui/badge";
    import { Loader2, Table as TableIcon } from "lucide-svelte";

    let { connectionId, tableName } = $props<{
        connectionId: string;
        tableName: string;
    }>();

    let structure = $state<ColumnDefinition[]>([]);
    let indexes = $state<IndexDefinition[]>([]);
    let loading = $state(false);
    let error = $state("");

    $effect(() => {
        if (connectionId && tableName) {
            untrack(() => loadStructure());
        }
    });

    async function loadStructure() {
        loading = true;
        error = "";
        structure = [];
        indexes = [];
        try {
            const [struct, idxs] = await Promise.all([
                getTableStructure(connectionId, tableName),
                getTableIndexes(connectionId, tableName),
            ]);
            structure = struct;
            indexes = idxs;
        } catch (e: any) {
            error = e.message || "Failed to load structure";
        } finally {
            loading = false;
        }
    }
</script>

<div class="h-full flex flex-col bg-background">
    <div class="p-4 border-b flex justify-between items-center bg-muted/10">
        <h2 class="text-lg font-semibold flex items-center gap-2">
            <span class="text-muted-foreground">Structure:</span>
            {tableName}
        </h2>
        <div class="flex gap-2">
            <Button
                variant="outline"
                size="sm"
                onclick={() => {
                    const tab = appState.tabs.find(
                        (t) => t.id === appState.activeTabId,
                    );
                    if (tab) {
                        tab.type = "data";
                    }
                }}
            >
                <TableIcon class="mr-2 h-4 w-4" />
                View Data
            </Button>
        </div>
    </div>

    <div class="flex-1 overflow-auto p-4">
        {#if loading}
            <div
                class="flex flex-col items-center justify-center h-full text-muted-foreground gap-2"
            >
                <Loader2 class="h-8 w-8 animate-spin" />
                <span>Loading structure...</span>
            </div>
        {:else if error}
            <div
                class="text-destructive p-4 border border-destructive/20 bg-destructive/10 rounded-md"
            >
                Error: {error}
            </div>
        {:else if structure.length === 0}
            <div
                class="flex items-center justify-center h-full text-muted-foreground"
            >
                No structure found
            </div>
        {:else}
            <div class="border rounded-md mb-8">
                <Table.Root>
                    <Table.Header>
                        <Table.Row>
                            <Table.Head>Column Name</Table.Head>
                            <Table.Head>Data Type</Table.Head>
                            <Table.Head>Nullable</Table.Head>
                            <Table.Head>Default</Table.Head>
                            <Table.Head>Foreign Key</Table.Head>
                            <Table.Head>Comment</Table.Head>
                        </Table.Row>
                    </Table.Header>
                    <Table.Body>
                        {#each structure as col}
                            <Table.Row>
                                <Table.Cell class="font-mono text-primary"
                                    >{col.column_name}</Table.Cell
                                >
                                <Table.Cell
                                    class="text-yellow-600 dark:text-yellow-400"
                                    >{col.data_type}</Table.Cell
                                >
                                <Table.Cell>
                                    <Badge
                                        variant={col.is_nullable === "YES"
                                            ? "secondary"
                                            : "destructive"}
                                        class="text-[10px]"
                                    >
                                        {col.is_nullable}
                                    </Badge>
                                </Table.Cell>
                                <Table.Cell class="text-muted-foreground"
                                    >{col.column_default || "-"}</Table.Cell
                                >
                                <Table.Cell
                                    class="text-purple-600 dark:text-purple-400"
                                    >{col.foreign_key || "-"}</Table.Cell
                                >
                                <Table.Cell class="text-muted-foreground italic"
                                    >{col.comment || "-"}</Table.Cell
                                >
                            </Table.Row>
                        {/each}
                    </Table.Body>
                </Table.Root>
            </div>

            {#if indexes.length > 0}
                <div class="mb-2">
                    <h3
                        class="text-md font-semibold text-muted-foreground mb-4"
                    >
                        Indexes
                    </h3>
                    <div class="border rounded-md">
                        <Table.Root>
                            <Table.Header>
                                <Table.Row>
                                    <Table.Head>Index Name</Table.Head>
                                    <Table.Head>Algorithm</Table.Head>
                                    <Table.Head>Unique</Table.Head>
                                    <Table.Head>Columns</Table.Head>
                                    <Table.Head>Condition</Table.Head>
                                    <Table.Head>Comment</Table.Head>
                                </Table.Row>
                            </Table.Header>
                            <Table.Body>
                                {#each indexes as idx}
                                    <Table.Row>
                                        <Table.Cell
                                            class="font-mono text-green-600 dark:text-green-400"
                                        >
                                            {idx.index_name}
                                            {#if idx.is_primary}
                                                <Badge
                                                    variant="default"
                                                    class="ml-2 text-[10px] h-4 px-1"
                                                    >PK</Badge
                                                >
                                            {/if}
                                        </Table.Cell>
                                        <Table.Cell
                                            class="text-muted-foreground"
                                            >{idx.index_algorithm}</Table.Cell
                                        >
                                        <Table.Cell>
                                            <Badge
                                                variant={idx.is_unique
                                                    ? "default"
                                                    : "secondary"}
                                                class="text-[10px]"
                                            >
                                                {idx.is_unique ? "YES" : "NO"}
                                            </Badge>
                                        </Table.Cell>
                                        <Table.Cell
                                            class="text-yellow-600 dark:text-yellow-400"
                                            >{idx.column_names}</Table.Cell
                                        >
                                        <Table.Cell
                                            class="text-muted-foreground"
                                            >{idx.condition || "-"}</Table.Cell
                                        >
                                        <Table.Cell
                                            class="text-muted-foreground italic"
                                            >{idx.comment || "-"}</Table.Cell
                                        >
                                    </Table.Row>
                                {/each}
                            </Table.Body>
                        </Table.Root>
                    </div>
                </div>
            {/if}
        {/if}
    </div>
</div>
