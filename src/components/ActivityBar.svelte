<script lang="ts">
    import { appState } from "$lib/state.svelte";
    import { disconnectDb } from "$lib/db";
    import { Plus, LogOut, Sun, Moon } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Separator } from "$lib/components/ui/separator";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import * as ContextMenu from "$lib/components/ui/context-menu";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import { cn } from "$lib/utils";
    import { setMode, mode } from "mode-watcher";

    function getInitials(name: string) {
        return name
            .split(" ")
            .map((n) => n[0])
            .join("")
            .substring(0, 2)
            .toUpperCase();
    }

    async function closeConnection(connectionId: string) {
        try {
            await disconnectDb(connectionId);
            appState.removeConnection(connectionId);
        } catch (e) {
            console.error("Failed to disconnect", e);
        }
    }
</script>

<div
    class="w-[70px] bg-muted/10 border-r flex flex-col items-center py-4 gap-4 shrink-0"
>
    <!-- New Connection / Home -->
    <Button
        variant={!appState.selectedConnectionId ? "default" : "ghost"}
        size="icon"
        class={cn(
            "w-12 h-12 rounded-xl transition-all duration-200",
            !appState.selectedConnectionId
                ? "bg-primary text-primary-foreground hover:bg-primary/90"
                : "hover:bg-muted",
        )}
        onclick={() => (appState.selectedConnectionId = null)}
        title="New Connection"
    >
        <Plus size={24} />
    </Button>

    <Separator class="w-8" />

    <!-- Active Connections -->
    <ScrollArea class="flex-1 w-full">
        <div class="flex flex-col items-center gap-3 w-full px-2">
            {#each appState.activeConnections as connection (connection.id)}
                <ContextMenu.Root>
                    <ContextMenu.Trigger>
                        <Button
                            variant={appState.selectedConnectionId ===
                            connection.id
                                ? "default"
                                : "ghost"}
                            size="icon"
                            class={cn(
                                "w-12 h-12 rounded-xl transition-all duration-200 relative group",
                                appState.selectedConnectionId === connection.id
                                    ? "bg-primary text-primary-foreground hover:bg-primary/90"
                                    : "bg-muted/50 hover:bg-muted",
                            )}
                            onclick={() =>
                                (appState.selectedConnectionId = connection.id)}
                            title={connection.config.name}
                        >
                            <span class="font-bold text-sm">
                                {getInitials(connection.config.name)}
                            </span>

                            <!-- Active Indicator -->
                            {#if appState.selectedConnectionId === connection.id}
                                <div
                                    class="absolute -left-3 top-1/2 -translate-y-1/2 w-1 h-8 bg-foreground rounded-r-full"
                                ></div>
                            {/if}
                        </Button>
                    </ContextMenu.Trigger>
                    <ContextMenu.Content>
                        <ContextMenu.Item
                            class="text-destructive focus:text-destructive"
                            onclick={() => closeConnection(connection.id)}
                        >
                            <LogOut class="mr-2 h-4 w-4" />
                            Close Connection
                        </ContextMenu.Item>
                    </ContextMenu.Content>
                </ContextMenu.Root>
            {/each}
        </div>
    </ScrollArea>

    <div class="mt-auto">
        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                <Button
                    variant="ghost"
                    size="icon"
                    class="w-10 h-10 rounded-full"
                >
                    <Sun
                        class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
                    />
                    <Moon
                        class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
                    />
                    <span class="sr-only">Toggle theme</span>
                </Button>
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
                <DropdownMenu.Item onclick={() => setMode("light")}>
                    Light
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => setMode("dark")}>
                    Dark
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={() => setMode("system")}>
                    System
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
</div>
