<script lang="ts">
  import { getAppState } from "$lib/stores/state.svelte";
  import { cn } from "$lib/utils";
  import {
    Database,
    LoaderCircle,
    TableProperties,
    Terminal,
    X,
  } from "lucide-svelte";
  import type { PageProps } from "./$types";
  import TableList from "./TableList.svelte";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import StructureViewer from "./StructureViewer.svelte";
  import DataViewer from "./DataViewer.svelte";
  import { TableTab } from "$lib/stores/table-tab.state.svelte";
  import SqlEditor from "./SqlEditor.svelte";
  import type { Action } from "svelte/action";
  import { toast } from "svelte-sonner";

  const appState = getAppState();
  const { data }: PageProps = $props();

  const space = $derived.by(() => {
    return appState.spaces.get(data.id);
  });

  const currentConnectionTabs = $derived(
    space
      ? (appState.tabs.filter((t) => t.connectionId === space.id) ?? [])
      : []
  );

  $effect(() => {
    if (space && space.status === "initial") {
      space.connect();
    }

    if (space && space.status === "error" && space.error) {
      toast.error(space.error, {
        duration: 3000,
        position: "bottom-right",
      });
    }
  });

  function closeTab(id: string, e: Event) {
    e.stopPropagation();
    if (!space) return;

    appState.closeTab(id);
  }

  function handleKeydown(e: KeyboardEvent) {
    // Check for Cmd+W (macOS) or Ctrl+W (Windows/Linux)
    if (e.key === "w" && (e.metaKey || e.ctrlKey) && space) {
      // Only close tab if there's an active tab

      e.preventDefault();
      appState.closeActiveTab(space.id);
    }
  }

  const scrolling: Action<HTMLButtonElement, string> = (node, tabId) => {
    $effect(() => {
      const scroll = tabId === space?.activeTabId;
      if (scroll) {
        node.scrollIntoView({ behavior: "smooth" });
      }
    });
  };
</script>

<svelte:window onkeydown={handleKeydown} />

{#if !space || space.status === "connecting"}
  <div
    class="flex-1 bg-background/80 backdrop-blur-sm flex items-center justify-center z-10"
  >
    <div class="flex flex-col items-center gap-2">
      <LoaderCircle class="h-8 w-8 animate-spin text-primary" />
      <span class="text-sm text-muted-foreground"
        >Connecting to database...</span
      >
    </div>
  </div>
{:else}
  <TableList {space} />

  <main class="flex-1 h-full overflow-hidden bg-background relative">
    <div class="h-full w-full flex flex-col bg-background">
      <!-- Second Level: Tab Bar (Only if a connection is selected) -->
      {#if space}
        <ScrollArea orientation="horizontal" class="bg-muted/10 border-b">
          <div class="flex">
            {#if currentConnectionTabs.length === 0}
              <div
                class="px-4 py-2 text-sm text-muted-foreground italic flex items-center h-10"
              >
                No open tabs
              </div>
            {/if}
            {#each currentConnectionTabs as tab (tab.id)}
              <button
                use:scrolling={tab.id}
                class={cn(
                  "group px-4 py-2 text-sm font-medium border-r flex items-center gap-2 min-w-[140px] max-w-60 h-10 transition-colors relative",
                  space?.activeTabId === tab.id
                    ? "bg-background text-foreground border-t-2 border-t-primary"
                    : "bg-muted/10 text-muted-foreground hover:bg-muted/30 border-t-2 border-t-transparent"
                )}
                onclick={() => {
                  if (space) {
                    space.activeTabId = tab.id;
                  }
                }}
              >
                {#if tab.type === "data"}
                  <Database class="h-4 w-4 text-blue-500" />
                {:else if tab.type === "query"}
                  <Terminal class="h-4 w-4 text-green-500" />
                {:else if tab.type === "structure"}
                  <TableProperties class="h-4 w-4 text-orange-500" />
                {/if}
                <span class="truncate flex-1 text-left">{tab.title}</span>
                <div
                  class={cn(
                    "opacity-0 group-hover:opacity-100 rounded-sm p-0.5 hover:bg-destructive/10 hover:text-destructive transition-all",
                    space?.activeTabId === tab.id && "opacity-100"
                  )}
                  onclick={(e) => closeTab(tab.id, e)}
                  role="button"
                  tabindex="0"
                  onkeydown={(e) => e.key === "Enter" && closeTab(tab.id, e)}
                >
                  <X class="h-3 w-3" />
                </div>
              </button>
            {/each}
          </div>
        </ScrollArea>
      {/if}

      <!-- Content Area -->
      <div class="flex-1 overflow-hidden relative">
        {#if !space.activeTabId}
          <div
            class="flex flex-col items-center justify-center h-full text-muted-foreground gap-4"
          >
            <div class="bg-muted/30 p-6 rounded-full">
              <Database class="h-16 w-16 opacity-20" />
            </div>
            <div class="text-center space-y-2">
              <h3 class="text-lg font-semibold text-foreground">
                No Active Tab
              </h3>
              <p class="text-sm max-w-xs mx-auto">
                Select a table from the sidebar or start a new SQL query.
              </p>
            </div>
            <!-- <Button onclick={createNewQuery} variant="outline" class="mt-2">
              <Terminal class="mr-2 h-4 w-4" />
              New Query
            </Button> -->
          </div>
        {:else}
          {#each currentConnectionTabs as tab, index (tab.id)}
            <div
              class="absolute inset-0 bg-background"
              class:hidden={tab.id !== space.activeTabId}
            >
              {#if tab.type === "data"}
                <DataViewer
                  tab={currentConnectionTabs[index] as TableTab<"data">}
                  {space}
                />
              {:else if tab.type === "structure"}
                <StructureViewer
                  tab={currentConnectionTabs[index] as TableTab<"structure">}
                  {space}
                />
              {:else if tab.type === "query"}
                <SqlEditor
                  connectionId={tab.connectionId}
                  tab={currentConnectionTabs[index] as TableTab<"query">}
                />
              {/if}
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </main>
{/if}
