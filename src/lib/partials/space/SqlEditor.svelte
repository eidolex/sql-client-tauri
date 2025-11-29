<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { EditorView, basicSetup } from "codemirror";
  import { sql, PostgreSQL } from "@codemirror/lang-sql";
  import { getDatabaseSchema, executeQuery, type QueryResult } from "$lib/db";
  import { Button } from "$lib/components/ui/button";
  import * as Resizable from "$lib/components/ui/resizable";
  import * as Table from "$lib/components/ui/table";
  import { Play, LoaderCircle } from "lucide-svelte";
  import { getAppState } from "$lib/state.svelte";

  const appState = getAppState();

  let { connectionId, initialQuery } = $props<{
    connectionId: string;
    initialQuery?: string;
  }>();

  let element: HTMLElement;
  let view: EditorView;
  let query = $state(
    initialQuery || "SELECT * FROM information_schema.tables;"
  );
  let result = $state<QueryResult>({ columns: [], rows: [] });
  let error = $state("");
  let loading = $state(false);

  onMount(async () => {
    let schema: Record<string, string[]> = {};
    try {
      const dbSchema = await getDatabaseSchema(connectionId);
      // Add * to each table's columns for autocomplete
      for (const [table, columns] of Object.entries(dbSchema)) {
        schema[table] = ["*", ...columns];
      }
      console.log("Loaded schema for autocomplete:", schema);
    } catch (e) {
      console.error("Failed to load schema for autocomplete", e);
    }

    view = new EditorView({
      doc: query,
      extensions: [
        basicSetup,
        sql({ schema, dialect: PostgreSQL }),
        EditorView.theme(
          {
            "&": { height: "100%", fontSize: "14px" },
            ".cm-scroller": { overflow: "auto" },
            ".cm-content": { fontFamily: "monospace" },
          },
          { dark: true }
        ),
      ],
      parent: element,
      dispatch: (tr) => {
        view.update([tr]);
        if (tr.docChanged) {
          query = view.state.doc.toString();
          const tab = appState.tabs.find((t) => t.id === appState.activeTabId);
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

<div class="h-full flex flex-col bg-background">
  <Resizable.PaneGroup
    direction="vertical"
    class="h-full w-full rounded-lg border"
  >
    <Resizable.Pane defaultSize={50}>
      <div class="h-full flex flex-col">
        <div class="p-2 bg-muted/10 border-b flex justify-between items-center">
          <span class="font-semibold text-sm text-muted-foreground"
            >SQL Editor</span
          >
          <Button size="sm" class="gap-2" onclick={runQuery} disabled={loading}>
            {#if loading}
              <LoaderCircle class="h-4 w-4 animate-spin" />
              <span>Running...</span>
            {:else}
              <Play class="h-4 w-4" />
              <span>Run</span>
            {/if}
          </Button>
        </div>
        <div
          class="flex-1 overflow-hidden bg-[#282c34]"
          bind:this={element}
        ></div>
      </div>
    </Resizable.Pane>
    <Resizable.Handle />
    <Resizable.Pane defaultSize={50}>
      <div class="h-full flex flex-col bg-background">
        <div
          class="p-2 bg-muted/10 border-b font-semibold text-sm text-muted-foreground"
        >
          Results
        </div>
        <div class="flex-1 overflow-auto p-4">
          {#if error}
            <div
              class="text-destructive p-4 border border-destructive/20 bg-destructive/10 rounded-md"
            >
              Error: {error}
            </div>
          {:else if result.rows.length === 0 && !loading}
            <div class="text-muted-foreground">No results</div>
          {:else if result.rows.length > 0}
            <div class="border rounded-md">
              <Table.Root>
                <Table.Header>
                  <Table.Row>
                    {#each result.columns as header}
                      <Table.Head>{header}</Table.Head>
                    {/each}
                  </Table.Row>
                </Table.Header>
                <Table.Body>
                  {#each result.rows as row}
                    <Table.Row>
                      {#each row as cell}
                        <Table.Cell
                          class="max-w-xs truncate"
                          title={String(cell)}
                        >
                          {String(cell)}
                        </Table.Cell>
                      {/each}
                    </Table.Row>
                  {/each}
                </Table.Body>
              </Table.Root>
            </div>
          {/if}
        </div>
      </div>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
