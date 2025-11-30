<script lang="ts">
  import { Database, Table, Terminal, LogOut } from "lucide-svelte";
  import { Button } from "$lib/components/ui/button";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { cn } from "$lib/utils";
  import { goto } from "$app/navigation";
  import type { WorkSpace } from "$lib/stores/work-space.state.svelte";
  import { getAppState } from "$lib/stores/state.svelte";

  const appState = getAppState();

  const { space }: { space: WorkSpace } = $props();

  function openTable(table: string) {
    appState.addTab({
      id: crypto.randomUUID(),
      title: table,
      connectionId: space.id,
      database: space.currentDatabase,
      type: "data",
      active: true,
      data: {
        table: table,
        page: 1,
        pageSize: 50,
        totalRows: 0,
      },
    });
  }

  function openSqlEditor(space: WorkSpace) {
    appState.addTab({
      id: crypto.randomUUID(),
      title: `Query: ${space.name}`,
      type: "query",
      connectionId: space.id,
      database: space.currentDatabase,
      active: true,
      data: {
        query: "",
      },
    });
  }

  async function switchDatabase(space: WorkSpace, database: string) {
    if (space.currentDatabase === database) {
      return;
    }

    const newConnection = {
      ...space.config,
      database: database,
    };
    const connectionId = await appState.addSpace(newConnection);
    goto(`/${connectionId}`);
  }

  async function closeCurrentConnection() {
    // if (space) {
    //   try {
    //     await disconnectDb(space.id);
    //     appState.removeSpace(space.id);
    //   } catch (e) {
    //     console.error("Failed to disconnect", e);
    //   }
    // }
  }
</script>

<div class="w-64 bg-muted/30 h-full flex flex-col border-r">
  <ScrollArea class="flex-1 min-h-0">
    <div class="p-2">
      <div
        class="text-xs text-muted-foreground uppercase font-semibold mb-2 px-2"
      >
        Databases
      </div>

      {#each space.databases as db}
        <div class="mb-1">
          <Button
            variant="ghost"
            class={cn(
              "w-full justify-start h-8 px-2 text-sm font-normal",
              space.currentDatabase === db
                ? "text-primary font-medium bg-primary/10 hover:bg-primary/20"
                : "text-muted-foreground",
            )}
            onclick={() => switchDatabase(space, db)}
          >
            <!-- {#if connectingDatabases.has(`${currentSpace.id}-${db}`)}
                <LoaderCircle class="mr-2 h-3.5 w-3.5 animate-spin" />
              {:else} -->
            <Database class="mr-2 h-3.5 w-3.5" />
            <!-- {/if} -->
            <span class="truncate flex-1 text-left">{db}</span>
            {#if space.currentDatabase === db}
              <div class="w-1.5 h-1.5 rounded-full bg-primary ml-auto"></div>
            {/if}
          </Button>

          <!-- Show tables if this is the active database -->
          {#if space.currentDatabase === db}
            <div class="ml-4 mt-1 border-l pl-2 space-y-0.5">
              <Button
                variant="ghost"
                class="w-full justify-start h-7 px-2 text-sm text-muted-foreground"
                onclick={() => openSqlEditor(space)}
              >
                <Terminal class="mr-2 h-3.5 w-3.5" />
                <span>SQL Editor</span>
              </Button>

              {#each space.tables as table}
                <Button
                  variant="ghost"
                  class="w-full justify-start h-7 px-2 text-sm text-muted-foreground font-normal"
                  onclick={() => openTable(table)}
                >
                  <Table class="mr-2 h-3.5 w-3.5 text-muted-foreground/70" />
                  <span class="truncate">{table}</span>
                </Button>
              {/each}
            </div>
          {/if}
        </div>
      {/each}
    </div>
  </ScrollArea>
</div>
