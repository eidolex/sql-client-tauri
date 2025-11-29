<script lang="ts">
  import {
    connectDb,
    saveConnection,
    loadConnections,
    deleteConnection,
    listDatabases,
    listTables,
    type SavedConnection,
  } from "$lib/db";
  import { appState } from "$lib/state.svelte";
  import { onMount } from "svelte";
  import {
    Trash2,
    Save,
    Plug,
    ChevronDown,
    ChevronRight,
    Database,
    Plus,
  } from "lucide-svelte";
  import { ask } from "@tauri-apps/plugin-dialog";

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { ScrollArea } from "$lib/components/ui/scroll-area";
  import { Separator } from "$lib/components/ui/separator";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import * as Select from "$lib/components/ui/select";
  import { cn } from "$lib/utils";

  let connections = $state<SavedConnection[]>([]);
  let selectedConnectionId = $state<string | null>(null);

  let currentConnection = $state<SavedConnection>({
    id: crypto.randomUUID(),
    name: "New Connection",
    host: "localhost",
    port: 5432,
    username: "postgres",
    password: "",
    database: "postgres",
    db_type: "postgres",
    ssh_enabled: false,
    ssh_host: "",
    ssh_port: 22,
    ssh_user: "",
    ssh_password: "",
    ssh_key_path: "",
  });

  let error = $state("");
  let loading = $state(false);
  let showSshConfig = $state(false);

  onMount(async () => {
    await loadSaved();
  });

  async function loadSaved() {
    try {
      connections = await loadConnections();
    } catch (e) {
      console.error("Failed to load connections", e);
    }
  }

  function selectConnection(conn: SavedConnection) {
    currentConnection = { ...conn };
    selectedConnectionId = conn.id;
    showSshConfig = conn.ssh_enabled;
  }

  function handleDbTypeChange() {
    if (currentConnection.db_type === "postgres") {
      currentConnection.port = 5432;
      currentConnection.username = "postgres";
      currentConnection.database = "postgres";
    } else if (currentConnection.db_type === "mysql") {
      currentConnection.port = 3306;
      currentConnection.username = "root";
      currentConnection.database = "mysql";
    }
  }

  function createNew() {
    currentConnection = {
      id: crypto.randomUUID(),
      name: "New Connection",
      host: "localhost",
      port: 5432,
      username: "postgres",
      password: "",
      database: "postgres",
      db_type: "postgres",
      ssh_enabled: false,
      ssh_host: "",
      ssh_port: 22,
      ssh_user: "",
      ssh_password: "",
      ssh_key_path: "",
    };
    selectedConnectionId = null;
    showSshConfig = false;
  }

  async function save() {
    try {
      await saveConnection(currentConnection);
      await loadSaved();
      selectedConnectionId = currentConnection.id;
    } catch (e: any) {
      error = e.message || "Failed to save connection";
    }
  }

  async function remove(id: string) {
    const confirmed = await ask(
      "Are you sure you want to delete this connection?",
      {
        title: "Delete Connection",
        kind: "warning",
      }
    );
    if (!confirmed) return;

    try {
      await deleteConnection(id);
      await loadSaved();
      if (selectedConnectionId === id) {
        createNew();
      }
    } catch (e: any) {
      error = e.message || "Failed to delete connection";
    }
  }

  async function connect() {
    loading = true;
    error = "";
    try {
      const connectionId = await connectDb(currentConnection);

      // Fetch initial metadata
      const databases = await listDatabases(connectionId);
      const tables = await listTables(connectionId);

      appState.addConnection({
        id: connectionId,
        config: { ...currentConnection },
        databases,
        tables,
        currentDatabase: currentConnection.database,
      });
    } catch (e: any) {
      error = e.message || "Failed to connect";
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex h-full bg-background text-foreground">
  <!-- Sidebar: Saved Connections -->
  <div class="w-64 border-r flex flex-col bg-muted/30">
    <div class="p-4 border-b flex justify-between items-center">
      <h3 class="font-semibold text-sm">Connections</h3>
      <Button variant="ghost" size="icon" onclick={createNew} class="h-8 w-8">
        <Plus class="h-4 w-4" />
      </Button>
    </div>
    <ScrollArea class="flex-1">
      <div class="p-2 space-y-1">
        {#each connections as conn}
          <div
            class={cn(
              "group flex justify-between items-center p-2 rounded-md cursor-pointer transition-colors",
              selectedConnectionId === conn.id
                ? "bg-primary/10 text-primary"
                : "hover:bg-muted"
            )}
            onclick={() => selectConnection(conn)}
            onkeydown={(e) => e.key === "Enter" && selectConnection(conn)}
            role="button"
            tabindex="0"
          >
            <div class="truncate text-sm font-medium flex-1">
              {conn.name}
              <div class="text-xs text-muted-foreground truncate">
                {conn.username}@{conn.host}:{conn.port}
              </div>
            </div>
            <button
              type="button"
              class="h-6 w-6 opacity-0 group-hover:opacity-100 text-muted-foreground hover:text-destructive inline-flex items-center justify-center rounded-md transition-colors hover:bg-accent"
              onclick={(e) => {
                e.stopPropagation();
                remove(conn.id);
              }}
            >
              <Trash2 class="h-3 w-3" />
            </button>
          </div>
        {/each}
      </div>
    </ScrollArea>
  </div>

  <!-- Main: Connection Form -->
  <div class="flex-1 flex flex-col overflow-y-auto bg-background">
    <div class="p-8 max-w-2xl mx-auto w-full">
      <div class="mb-6 flex items-center gap-2">
        <div class="p-2 bg-primary/10 rounded-lg">
          <Plug class="text-primary h-6 w-6" />
        </div>
        <h2 class="text-2xl font-bold tracking-tight">
          {currentConnection.name || "New Connection"}
        </h2>
      </div>

      <div class="space-y-6">
        <!-- General Settings -->
        <Card>
          <CardHeader>
            <CardTitle>General Settings</CardTitle>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="grid gap-2">
              <Label for="connection_name">Connection Name</Label>
              <Input
                id="connection_name"
                bind:value={currentConnection.name}
                placeholder="My Database"
                autocapitalize="none"
                autocorrect="off"
              />
            </div>

            <div class="grid gap-2">
              <Label>Database Type</Label>
              <div class="grid grid-cols-2 gap-4">
                <Button
                  variant="outline"
                  class={cn(
                    "h-auto flex-col gap-2 p-4",
                    currentConnection.db_type === "postgres" &&
                      "border-primary bg-primary/5"
                  )}
                  onclick={() => {
                    currentConnection.db_type = "postgres";
                    handleDbTypeChange();
                  }}
                >
                  <Database class="h-6 w-6" />
                  <span>PostgreSQL</span>
                </Button>
                <Button
                  variant="outline"
                  class={cn(
                    "h-auto flex-col gap-2 p-4",
                    currentConnection.db_type === "mysql" &&
                      "border-primary bg-primary/5"
                  )}
                  onclick={() => {
                    currentConnection.db_type = "mysql";
                    handleDbTypeChange();
                  }}
                >
                  <Database class="h-6 w-6" />
                  <span>MySQL</span>
                </Button>
              </div>
            </div>

            <div class="grid grid-cols-3 gap-4">
              <div class="col-span-2 grid gap-2">
                <Label for="connection_host">Host</Label>
                <Input
                  id="connection_host"
                  bind:value={currentConnection.host}
                  placeholder="localhost"
                  autocapitalize="none"
                  autocorrect="off"
                />
              </div>
              <div class="grid gap-2">
                <Label for="connection_port">Port</Label>
                <Input
                  id="connection_port"
                  type="number"
                  bind:value={currentConnection.port}
                  placeholder="5432"
                  autocapitalize="none"
                  autocorrect="off"
                />
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="grid gap-2">
                <Label for="connection_username">Username</Label>
                <Input
                  id="connection_username"
                  bind:value={currentConnection.username}
                  placeholder="postgres"
                  autocapitalize="none"
                  autocorrect="off"
                />
              </div>
              <div class="grid gap-2">
                <Label for="connection_password">Password</Label>
                <Input
                  id="connection_password"
                  type="password"
                  bind:value={currentConnection.password}
                  placeholder="••••••••"
                  autocapitalize="none"
                  autocorrect="off"
                />
              </div>
            </div>

            <div class="grid gap-2">
              <Label for="connection_database">Database</Label>
              <Input
                id="connection_database"
                bind:value={currentConnection.database}
                placeholder="postgres"
                autocapitalize="none"
                autocorrect="off"
              />
            </div>
          </CardContent>
        </Card>

        <!-- SSH Tunnel Settings -->
        <Card>
          <button
            class="w-full flex justify-between items-center p-6 text-left"
            onclick={() => {
              showSshConfig = !showSshConfig;
              if (showSshConfig) currentConnection.ssh_enabled = true;
              else currentConnection.ssh_enabled = false;
            }}
          >
            <div class="flex items-center gap-2">
              <!-- Checkbox needs to be handled carefully inside a button -->
              <div
                class="flex items-center space-x-2"
                onclick={(e) => e.stopPropagation()}
                onkeydown={(e) => e.stopPropagation()}
                role="button"
                tabindex="0"
              >
                <Checkbox
                  id="ssh_enabled"
                  checked={currentConnection.ssh_enabled}
                  onCheckedChange={(v: boolean) =>
                    (currentConnection.ssh_enabled = v)}
                />
                <Label
                  for="ssh_enabled"
                  class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                >
                  Use SSH Tunnel
                </Label>
              </div>
            </div>
            {#if showSshConfig}
              <ChevronDown class="h-4 w-4 text-muted-foreground" />
            {:else}
              <ChevronRight class="h-4 w-4 text-muted-foreground" />
            {/if}
          </button>

          {#if showSshConfig}
            <div class="px-6 pb-6 space-y-4 border-t pt-4">
              <div class="grid grid-cols-3 gap-4">
                <div class="col-span-2 grid gap-2">
                  <Label for="ssh_host">SSH Host</Label>
                  <Input
                    id="ssh_host"
                    bind:value={currentConnection.ssh_host}
                    placeholder="remote.server.com"
                    autocapitalize="none"
                    autocorrect="off"
                  />
                </div>
                <div class="grid gap-2">
                  <Label for="ssh_port">SSH Port</Label>
                  <Input
                    id="ssh_port"
                    type="number"
                    bind:value={currentConnection.ssh_port}
                    placeholder="22"
                    autocapitalize="none"
                    autocorrect="off"
                  />
                </div>
              </div>
              <div class="grid grid-cols-3 gap-4">
                <div class="grid gap-2">
                  <Label for="ssh_user">SSH User</Label>
                  <Input
                    id="ssh_user"
                    bind:value={currentConnection.ssh_user}
                    placeholder="root"
                    autocapitalize="none"
                    autocorrect="off"
                  />
                </div>
                <div class="grid gap-2">
                  <Label for="ssh_password">SSH Password</Label>
                  <Input
                    id="ssh_password"
                    type="password"
                    bind:value={currentConnection.ssh_password}
                    placeholder="••••••••"
                    autocapitalize="none"
                    autocorrect="off"
                  />
                </div>
                <div class="grid gap-2">
                  <Label for="ssh_key_path">SSH Key Path</Label>
                  <Input
                    id="ssh_key_path"
                    bind:value={currentConnection.ssh_key_path}
                    placeholder="~/.ssh/id_rsa"
                    autocapitalize="none"
                    autocorrect="off"
                  />
                </div>
              </div>
            </div>
          {/if}
        </Card>

        {#if error}
          <div
            class="text-destructive text-sm p-3 bg-destructive/10 rounded-md border border-destructive/20"
          >
            {error}
          </div>
        {/if}

        <div class="flex gap-4 pt-4">
          <Button onclick={connect} disabled={loading} class="flex-1 gap-2">
            {#if loading}
              Connecting...
            {:else}
              <Plug class="h-4 w-4" /> Connect
            {/if}
          </Button>
          <Button variant="secondary" onclick={save} class="gap-2">
            <Save class="h-4 w-4" /> Save
          </Button>
        </div>
      </div>
    </div>
  </div>
</div>
