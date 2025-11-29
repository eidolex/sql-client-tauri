<script lang="ts">
    import { untrack } from "svelte";
    import { appState, type Tab } from "$lib/state.svelte";
    import { getTableData, getTableStructure, type QueryResult } from "$lib/db";
    import {
        Filter as FilterIcon,
        RefreshCw,
        Settings2,
        Plus,
        Trash2,
        ChevronLeft,
        ChevronRight,
        ArrowUp,
        ArrowDown,
        Loader2,
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import * as Select from "$lib/components/ui/select";
    import * as Table from "$lib/components/ui/table";
    import { Badge } from "$lib/components/ui/badge";
    import { Card } from "$lib/components/ui/card";
    import { ScrollArea } from "$lib/components/ui/scroll-area";

    let { tab } = $props<{ tab: Tab }>();

    let loading = $state(false);
    let error = $state("");
    let showFilters = $state(false);

    interface Filter {
        field: string;
        operator: string;
        value: string;
    }

    interface Sort {
        field: string;
        order: "ASC" | "DESC";
    }

    let filters = $state<Filter[]>([]);
    let sorts = $state<Sort[]>([]);

    // Initialize defaults if missing
    if (!tab.page) tab.page = 1;
    if (!tab.pageSize) tab.pageSize = 50;
    // Don't initialize tab.data to [] here, so we can distinguish "not loaded" (undefined) from "loaded but empty" ([])
    if (!tab.columns) tab.columns = [];
    if (tab.totalRows === undefined) tab.totalRows = 0;

    let totalPages = $derived(
        Math.ceil((tab.totalRows || 0) / (tab.pageSize || 50)) || 1,
    );

    $effect(() => {
        if (tab.table && (!tab.columns || tab.columns.length === 0)) {
            untrack(() => loadStructure());
        }

        // Load data if empty or if explicit refresh needed (we can add a timestamp later if needed)
        // Only load if data is undefined (never loaded)
        if (tab.data === undefined && !loading && !error) {
            untrack(() => loadData());
        }
    });

    async function loadStructure() {
        if (!tab.table) return;
        try {
            const structure = await getTableStructure(
                tab.connectionId,
                tab.table,
            );
            // The structure returns objects with column_name, data_type, etc.
            // We just need the names for the headers for now.
            // Based on get_table_structure implementation:
            // SELECT column_name, data_type, is_nullable ...
            tab.columns = structure.map((col: any) => col.column_name);
        } catch (e) {
            console.error("Failed to load structure", e);
        }
    }

    function handlePageChange(newPage: number) {
        if (newPage >= 1 && newPage <= totalPages) {
            tab.page = newPage;
            loadData();
        }
    }

    async function loadData() {
        if (!tab.table) return;

        loading = true;
        error = "";
        try {
            const offset = ((tab.page || 1) - 1) * (tab.pageSize || 50);
            const result = await getTableData(
                tab.connectionId,
                tab.table,
                tab.pageSize || 50,
                offset,
                filters,
                sorts,
            );

            tab.data = result.rows;
            // Only update columns if we don't have them yet, or if the result has them
            // But usually we trust loadStructure for the initial view.
            // However, getTableData returns columns too. Let's ensure they match or just update.
            if (!tab.columns || tab.columns.length === 0) {
                tab.columns = result.columns;
            }
            tab.totalRows = result.total_rows || 0;
        } catch (e: any) {
            error = e.message || "Failed to load data";
        } finally {
            loading = false;
        }
    }

    function formatCell(value: any): string {
        if (value === null || value === undefined) {
            return "";
        }
        if (typeof value === "object") {
            return JSON.stringify(value);
        }
        return String(value);
    }

    function addFilter() {
        if (tab.columns && tab.columns.length > 0) {
            filters.push({
                field: tab.columns[0],
                operator: "=",
                value: "",
            });
        }
    }

    function removeFilter(index: number) {
        filters.splice(index, 1);
    }

    function applyFilters() {
        tab.page = 1; // Reset to first page when filtering
        loadData();
    }

    function handleHeaderClick(field: string, event: MouseEvent) {
        const existingSortIndex = sorts.findIndex((s) => s.field === field);
        const isShift = event.shiftKey;

        if (isShift) {
            if (existingSortIndex !== -1) {
                // Toggle existing
                if (sorts[existingSortIndex].order === "ASC") {
                    sorts[existingSortIndex].order = "DESC";
                } else {
                    sorts.splice(existingSortIndex, 1);
                }
            } else {
                // Add new
                sorts.push({ field, order: "ASC" });
            }
        } else {
            if (existingSortIndex !== -1 && sorts.length === 1) {
                // Toggle existing single sort
                if (sorts[0].order === "ASC") {
                    sorts[0].order = "DESC";
                } else {
                    sorts = [];
                }
            } else {
                // Replace with new single sort
                sorts = [{ field, order: "ASC" }];
            }
        }
        loadData();
    }

    function getSortIndicator(field: string) {
        const index = sorts.findIndex((s) => s.field === field);
        if (index === -1) return null;
        return {
            order: sorts[index].order,
            index: sorts.length > 1 ? index + 1 : null,
        };
    }
</script>

<div class="h-full flex flex-col bg-background">
    <div class="p-4 border-b flex flex-col gap-4 bg-muted/10">
        <div class="flex justify-between items-center">
            <h2 class="text-lg font-semibold flex items-center gap-2">
                <span class="text-muted-foreground">Table:</span>
                {tab.table}
            </h2>
            <div class="flex gap-2">
                <Button
                    variant={showFilters ? "secondary" : "outline"}
                    size="sm"
                    class="gap-2"
                    onclick={() => (showFilters = !showFilters)}
                >
                    <FilterIcon class="h-4 w-4" />
                    Filter
                    {#if filters.length > 0}
                        <Badge
                            variant="secondary"
                            class="ml-1 px-1.5 py-0 h-5 text-[10px]"
                            >{filters.length}</Badge
                        >
                    {/if}
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    class="gap-2"
                    onclick={loadData}
                >
                    <RefreshCw
                        class={loading ? "h-4 w-4 animate-spin" : "h-4 w-4"}
                    />
                    Refresh
                </Button>
                <Button
                    variant="outline"
                    size="sm"
                    onclick={() => {
                        tab.type = "structure";
                    }}
                >
                    Structure
                </Button>
            </div>
        </div>

        {#if showFilters}
            <Card class="p-4 bg-muted/30">
                <div class="flex flex-col gap-2">
                    {#each filters as filter, i}
                        <div class="flex gap-2 items-center">
                            <div class="w-48">
                                <select
                                    class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                                    bind:value={filter.field}
                                >
                                    {#each tab.columns || [] as col}
                                        <option value={col}>{col}</option>
                                    {/each}
                                </select>
                            </div>
                            <div class="w-32">
                                <select
                                    class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
                                    bind:value={filter.operator}
                                >
                                    <option value="=">=</option>
                                    <option value=">=">&gt;=</option>
                                    <option value="<=">&lt;=</option>
                                    <option value=">">&gt;</option>
                                    <option value="<">&lt;</option>
                                    <option value="contain">contain</option>
                                    <option value="start with"
                                        >start with</option
                                    >
                                    <option value="end with">end with</option>
                                    <option value="not null">not null</option>
                                    <option value="is null">is null</option>
                                </select>
                            </div>
                            {#if filter.operator !== "not null" && filter.operator !== "is null"}
                                <Input
                                    type="text"
                                    class="flex-1 h-9"
                                    placeholder="Value"
                                    bind:value={filter.value}
                                />
                            {/if}
                            <Button
                                variant="ghost"
                                size="icon"
                                class="h-9 w-9 text-muted-foreground hover:text-destructive"
                                onclick={() => removeFilter(i)}
                            >
                                <Trash2 class="h-4 w-4" />
                            </Button>
                        </div>
                    {/each}
                    <div class="flex gap-2 mt-2">
                        <Button
                            variant="ghost"
                            size="sm"
                            class="text-primary hover:text-primary/80 gap-1"
                            onclick={addFilter}
                        >
                            <Plus class="h-4 w-4" />
                            Add Filter
                        </Button>
                        <div class="flex-1"></div>
                        <Button size="sm" onclick={applyFilters}>Apply</Button>
                    </div>
                </div>
            </Card>
        {/if}
    </div>

    <div class="flex-1 overflow-hidden flex flex-col relative">
        <div class="flex-1 overflow-auto">
            {#if error}
                <div
                    class="p-4 m-4 text-destructive bg-destructive/10 border border-destructive/20 rounded-md"
                >
                    Error: {error}
                </div>
            {:else if !tab.columns || tab.columns.length === 0}
                {#if loading}
                    <div
                        class="flex flex-col items-center justify-center h-full text-muted-foreground gap-2"
                    >
                        <Loader2 class="h-8 w-8 animate-spin" />
                        <span>Loading structure...</span>
                    </div>
                {:else}
                    <div
                        class="flex items-center justify-center h-full text-muted-foreground"
                    >
                        No structure found
                    </div>
                {/if}
            {:else}
                <div class="min-w-full inline-block align-middle">
                    <Table.Root>
                        <Table.Header
                            class="sticky top-0 bg-background z-10 shadow-sm"
                        >
                            <Table.Row>
                                {#each tab.columns || [] as header}
                                    {@const sort = getSortIndicator(header)}
                                    <Table.Head
                                        class="cursor-pointer hover:bg-muted/50 select-none whitespace-nowrap"
                                        onclick={(e) =>
                                            handleHeaderClick(header, e)}
                                    >
                                        <div class="flex items-center gap-1">
                                            {header}
                                            {#if sort}
                                                <span
                                                    class="text-primary flex items-center"
                                                >
                                                    {#if sort.order === "ASC"}
                                                        <ArrowUp
                                                            class="h-3 w-3"
                                                        />
                                                    {:else}
                                                        <ArrowDown
                                                            class="h-3 w-3"
                                                        />
                                                    {/if}
                                                    {#if sort.index}
                                                        <span
                                                            class="ml-0.5 text-[10px]"
                                                            >{sort.index}</span
                                                        >
                                                    {/if}
                                                </span>
                                            {/if}
                                        </div>
                                    </Table.Head>
                                {/each}
                            </Table.Row>
                        </Table.Header>
                        <Table.Body>
                            {#if loading && (!tab.data || tab.data.length === 0)}
                                <Table.Row>
                                    <Table.Cell
                                        colspan={tab.columns.length}
                                        class="h-24 text-center"
                                    >
                                        <div
                                            class="flex justify-center items-center gap-2 text-muted-foreground"
                                        >
                                            <Loader2
                                                class="h-4 w-4 animate-spin"
                                            />
                                            Loading data...
                                        </div>
                                    </Table.Cell>
                                </Table.Row>
                            {:else if !tab.data || tab.data.length === 0}
                                <Table.Row>
                                    <Table.Cell
                                        colspan={tab.columns.length}
                                        class="h-24 text-center text-muted-foreground"
                                    >
                                        No data found
                                    </Table.Cell>
                                </Table.Row>
                            {:else}
                                {#each tab.data || [] as row}
                                    <Table.Row>
                                        {#each row as cell}
                                            <Table.Cell
                                                class="max-w-xs truncate"
                                                title={formatCell(cell)}
                                            >
                                                {formatCell(cell)}
                                            </Table.Cell>
                                        {/each}
                                    </Table.Row>
                                {/each}
                            {/if}
                        </Table.Body>
                    </Table.Root>
                </div>

                {#if loading && tab.data && tab.data.length > 0}
                    <div
                        class="absolute inset-0 bg-background/50 flex items-center justify-center z-20 backdrop-blur-[1px]"
                    >
                        <div
                            class="bg-background border shadow-lg px-4 py-2 rounded-md flex items-center gap-2"
                        >
                            <Loader2 class="h-4 w-4 animate-spin" />
                            <span>Loading...</span>
                        </div>
                    </div>
                {/if}
            {/if}
        </div>

        <!-- Pagination Controls -->
        {#if !error && (tab.columns?.length || 0) > 0}
            <div
                class="flex justify-between items-center p-4 border-t bg-muted/10 text-sm text-muted-foreground"
            >
                <div>
                    Showing {((tab.page || 1) - 1) * (tab.pageSize || 50) + 1} to
                    {Math.min(
                        (tab.page || 1) * (tab.pageSize || 50),
                        tab.totalRows || 0,
                    )} of {tab.totalRows} rows
                </div>
                <div class="flex gap-2 items-center">
                    <Button
                        variant="outline"
                        size="sm"
                        disabled={tab.page === 1}
                        onclick={() => handlePageChange((tab.page || 1) - 1)}
                    >
                        <ChevronLeft class="h-4 w-4 mr-1" />
                        Previous
                    </Button>
                    <span class="min-w-[80px] text-center"
                        >Page {tab.page} of {totalPages}</span
                    >
                    <Button
                        variant="outline"
                        size="sm"
                        disabled={tab.page === totalPages}
                        onclick={() => handlePageChange((tab.page || 1) + 1)}
                    >
                        Next
                        <ChevronRight class="h-4 w-4 ml-1" />
                    </Button>
                </div>
            </div>
        {/if}
    </div>
</div>
