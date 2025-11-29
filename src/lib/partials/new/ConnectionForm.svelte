<script lang="ts">
  import { goto } from "$app/navigation";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { type SavedConnection } from "$lib/db";
  import { getAppState } from "$lib/stores/state.svelte";
  import { getConnectionState } from "$lib/stores/connection.state.svelte";
  import { Database, Plug, Save } from "lucide-svelte";
  import { cn } from "tailwind-variants";

  const appState = getAppState();
  const connectionState = getConnectionState();

  let error = $state("");
  let loading = $state(false);
  let formData = $state<SavedConnection>(newConnection());

  $effect(() => {
    const selected = connectionState.connections.find(
      (c) => c.id === connectionState.selectedConnectionId,
    );
    if (selected) {
      formData = { ...selected };
    } else {
      formData = newConnection();
    }
  });

  function newConnection(): SavedConnection {
    return {
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
  }

  function onDbTypeChange() {
    if (formData.db_type === "postgres") {
      formData.port = 5432;
      if (formData.database === "mysql") {
        formData.database = "postgres";
      }
    } else if (formData.db_type === "mysql") {
      formData.port = 3306;
      if (formData.database === "postgres") {
        formData.database = "mysql";
      }
    }
  }

  async function onSave(e: SubmitEvent) {
    e.preventDefault();
    try {
      error = "";
      await connectionState.save(formData);
    } catch (e: any) {
      error = e.message || "Failed to save connection";
    }
  }

  async function connect() {
    loading = true;
    error = "";
    try {
      const connectionId = appState.getConnectionId(formData);
      appState.addSpace({ ...formData });
      goto(`/${connectionId}`);
    } catch (e: any) {
      error = e.message || "Failed to connect";
    } finally {
      loading = false;
    }
  }
</script>

<div class="p-8 max-w-2xl mx-auto w-full">
  <div class="mb-6 flex items-center gap-2">
    <div class="p-2 bg-primary/10 rounded-lg">
      <Plug class="text-primary h-6 w-6" />
    </div>
    <h2 class="text-2xl font-bold tracking-tight">
      {formData.name || "New Connection"}
    </h2>
  </div>
  <form method="post" class="space-y-6" onsubmit={onSave}>
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
            bind:value={formData.name}
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
                formData.db_type === "postgres" &&
                  "border-primary bg-primary/5",
              )}
              onclick={() => {
                formData.db_type = "postgres";
                onDbTypeChange();
              }}
            >
              <Database class="h-6 w-6" />
              <span>PostgreSQL</span>
            </Button>
            <Button
              variant="outline"
              class={cn(
                "h-auto flex-col gap-2 p-4",
                formData.db_type === "mysql" && "border-primary bg-primary/5",
              )}
              onclick={() => {
                formData.db_type = "mysql";
                onDbTypeChange();
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
              bind:value={formData.host}
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
              bind:value={formData.port}
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
              bind:value={formData.username}
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
              bind:value={formData.password}
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
            bind:value={formData.database}
            placeholder="postgres"
            autocapitalize="none"
            autocorrect="off"
          />
        </div>
      </CardContent>
    </Card>

    <!-- SSH Tunnel Settings -->
    <Card>
      <div class="flex items-center gap-2 px-6">
        <!-- Checkbox needs to be handled carefully inside a button -->
        <Checkbox
          id="ssh_enabled"
          checked={formData.ssh_enabled}
          onCheckedChange={(v: boolean) => (formData.ssh_enabled = v)}
        />
        <Label
          for="ssh_enabled"
          class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
        >
          Use SSH Tunnel
        </Label>
      </div>

      {#if formData.ssh_enabled}
        <div class="px-6 pb-6 space-y-4 border-t pt-4">
          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-2 grid gap-2">
              <Label for="ssh_host">SSH Host</Label>
              <Input
                id="ssh_host"
                bind:value={formData.ssh_host}
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
                bind:value={formData.ssh_port}
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
                bind:value={formData.ssh_user}
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
                bind:value={formData.ssh_password}
                placeholder="••••••••"
                autocapitalize="none"
                autocorrect="off"
              />
            </div>
            <div class="grid gap-2">
              <Label for="ssh_key_path">SSH Key Path</Label>
              <Input
                id="ssh_key_path"
                bind:value={formData.ssh_key_path}
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
      <Button
        type="button"
        class="flex-1 gap-2"
        onclick={connect}
        disabled={loading}
      >
        {#if loading}
          Connecting...
        {:else}
          <Plug class="h-4 w-4" /> Connect
        {/if}
      </Button>
      <Button type="submit" variant="secondary" class="gap-2">
        <Save class="h-4 w-4" /> Save
      </Button>
    </div>
  </form>
</div>
