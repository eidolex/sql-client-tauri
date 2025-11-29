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
  } from "lucide-svelte";

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
    if (!confirm("Are you sure you want to delete this connection?")) return;
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

<div class="flex h-full bg-gray-950 text-white">
  <!-- Sidebar: Saved Connections -->
  <div class="w-64 border-r border-gray-800 flex flex-col bg-gray-900">
    <div class="p-4 border-b border-gray-800 flex justify-between items-center">
      <h3 class="font-bold text-gray-300">Connections</h3>
      <button
        class="text-sm bg-blue-600 hover:bg-blue-500 px-2 py-1 rounded"
        onclick={createNew}
      >
        New
      </button>
    </div>
    <div class="flex-1 overflow-y-auto p-2 space-y-1">
      {#each connections as conn}
        <div
          class="group flex justify-between items-center p-2 rounded cursor-pointer {selectedConnectionId ===
          conn.id
            ? 'bg-blue-900/30 border border-blue-800'
            : 'hover:bg-gray-800 border border-transparent'}"
          onclick={() => selectConnection(conn)}
          onkeydown={(e) => e.key === "Enter" && selectConnection(conn)}
          role="button"
          tabindex="0"
        >
          <div class="truncate text-sm font-medium">
            {conn.name}
            <div class="text-xs text-gray-500 truncate">
              {conn.username}@{conn.host}:{conn.port}
            </div>
          </div>
          <button
            class="opacity-0 group-hover:opacity-100 p-1 hover:text-red-400 transition-opacity"
            onclick={(e) => {
              e.stopPropagation();
              remove(conn.id);
            }}
          >
            <Trash2 size={14} />
          </button>
        </div>
      {/each}
    </div>
  </div>

  <!-- Main: Connection Form -->
  <div class="flex-1 flex flex-col overflow-y-auto">
    <div class="p-8 max-w-2xl mx-auto w-full">
      <h2 class="text-2xl font-bold mb-6 flex items-center gap-2">
        <Plug class="text-blue-500" />
        {currentConnection.name || "New Connection"}
      </h2>

      <div class="space-y-6">
        <!-- General Settings -->
        <div class="space-y-4">
          <div>
            <label
              for="id_connection_name"
              class="block text-sm font-medium text-gray-400 mb-1"
              >Connection Name</label
            >
            <input
              id="id_connection_name"
              type="text"
              bind:value={currentConnection.name}
              class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
              placeholder="My Database"
              autocapitalize="off"
              autocomplete="off"
              spellcheck="false"
              autocorrect="off"
            />
          </div>

          <div>
            <label
              for="id_connection_type"
              class="block text-sm font-medium text-gray-400 mb-1"
              >Database Type</label
            >
            <div class="grid grid-cols-2 gap-4">
              <button
                class="flex flex-col items-center justify-center p-4 rounded-lg border-2 transition-all {currentConnection.db_type ===
                'postgres'
                  ? 'border-blue-500 bg-blue-900/20 text-blue-400'
                  : 'border-gray-700 bg-gray-900 text-gray-400 hover:border-gray-600 hover:bg-gray-800'}"
                onclick={() => {
                  currentConnection.db_type = "postgres";
                  handleDbTypeChange();
                }}
              >
                <Database class="w-8 h-8 mb-2" />
                <span class="font-medium">PostgreSQL</span>
              </button>

              <button
                class="flex flex-col items-center justify-center p-4 rounded-lg border-2 transition-all {currentConnection.db_type ===
                'mysql'
                  ? 'border-orange-500 bg-orange-900/20 text-orange-400'
                  : 'border-gray-700 bg-gray-900 text-gray-400 hover:border-gray-600 hover:bg-gray-800'}"
                onclick={() => {
                  currentConnection.db_type = "mysql";
                  handleDbTypeChange();
                }}
              >
                <Database class="w-8 h-8 mb-2" />
                <span class="font-medium">MySQL</span>
              </button>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-4">
            <div class="col-span-2">
              <label
                for="id_connection_host"
                class="block text-sm font-medium text-gray-400 mb-1">Host</label
              >
              <input
                id="id_connection_host"
                type="text"
                bind:value={currentConnection.host}
                class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                placeholder="localhost"
                autocapitalize="off"
                autocomplete="off"
                spellcheck="false"
                autocorrect="off"
              />
            </div>
            <div>
              <label
                for="id_connection_port"
                class="block text-sm font-medium text-gray-400 mb-1">Port</label
              >
              <input
                id="id_connection_port"
                type="number"
                bind:value={currentConnection.port}
                class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                placeholder="5432"
                autocapitalize="off"
                autocomplete="off"
                spellcheck="false"
                autocorrect="off"
              />
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div>
              <label
                for="id_connection_username"
                class="block text-sm font-medium text-gray-400 mb-1"
                >Username</label
              >
              <input
                id="id_connection_username"
                type="text"
                bind:value={currentConnection.username}
                class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                placeholder="postgres"
                autocapitalize="off"
                autocomplete="off"
                spellcheck="false"
                autocorrect="off"
              />
            </div>
            <div>
              <label
                for="id_connection_password"
                class="block text-sm font-medium text-gray-400 mb-1"
                >Password</label
              >
              <input
                id="id_connection_password"
                type="password"
                bind:value={currentConnection.password}
                class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                placeholder="••••••••"
                autocapitalize="off"
                autocomplete="off"
                spellcheck="false"
                autocorrect="off"
              />
            </div>
          </div>

          <div>
            <label
              for="id_connection_database"
              class="block text-sm font-medium text-gray-400 mb-1"
              >Database</label
            >
            <input
              id="id_connection_database"
              type="text"
              bind:value={currentConnection.database}
              class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
              placeholder="postgres"
              autocapitalize="off"
              autocomplete="off"
              spellcheck="false"
              autocorrect="off"
            />
          </div>
        </div>

        <!-- SSH Tunnel Settings -->
        <div class="border border-gray-800 rounded bg-gray-900/30">
          <button
            id="id_connection_ssh_toggle"
            class="w-full flex justify-between items-center p-4 text-left font-medium text-gray-300 hover:bg-gray-800/50 transition-colors"
            onclick={() => {
              showSshConfig = !showSshConfig;
              if (showSshConfig) currentConnection.ssh_enabled = true;
              else currentConnection.ssh_enabled = false;
            }}
          >
            <div class="flex items-center gap-2">
              <input
                id="id_connection_ssh_checkbox"
                type="checkbox"
                checked={currentConnection.ssh_enabled}
                onclick={(e) => e.stopPropagation()}
                onchange={(e) =>
                  (currentConnection.ssh_enabled = e.currentTarget.checked)}
                class="rounded border-gray-700 bg-gray-900 text-blue-600 focus:ring-blue-500"
              />
              <span>Use SSH Tunnel</span>
            </div>
            {#if showSshConfig}
              <ChevronDown size={16} />
            {:else}
              <ChevronRight size={16} />
            {/if}
          </button>

          {#if showSshConfig}
            <div class="p-4 pt-0 space-y-4 border-t border-gray-800 mt-2">
              <div class="grid grid-cols-3 gap-4">
                <div class="col-span-2">
                  <label
                    for="id_connection_ssh_host"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >SSH Host</label
                  >
                  <input
                    id="id_connection_ssh_host"
                    type="text"
                    bind:value={currentConnection.ssh_host}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                    placeholder="remote.server.com"
                    autocapitalize="off"
                    autocomplete="off"
                    spellcheck="false"
                    autocorrect="off"
                  />
                </div>
                <div>
                  <label
                    for="id_connection_ssh_port"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >SSH Port</label
                  >
                  <input
                    id="id_connection_ssh_port"
                    type="number"
                    bind:value={currentConnection.ssh_port}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                    placeholder="22"
                    autocapitalize="off"
                    autocomplete="off"
                    spellcheck="false"
                    autocorrect="off"
                  />
                </div>

                <div>
                  <label
                    for="id_connection_ssh_password"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >SSH Password</label
                  >
                  <input
                    id="id_connection_ssh_password"
                    type="password"
                    bind:value={currentConnection.ssh_password}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                    placeholder="••••••••"
                    autocapitalize="off"
                    autocomplete="off"
                    spellcheck="false"
                    autocorrect="off"
                  />
                </div>
              </div>

              <div class="grid grid-cols-2 gap-4">
                <div>
                  <label
                    for="id_connection_ssh_user"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >SSH User</label
                  >
                  <input
                    id="id_connection_ssh_user"
                    type="text"
                    bind:value={currentConnection.ssh_user}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                    placeholder="root"
                    autocapitalize="off"
                    autocomplete="off"
                    spellcheck="false"
                    autocorrect="off"
                  />
                </div>
                <div>
                  <label
                    for="id_connection_ssh_key_path"
                    class="block text-sm font-medium text-gray-400 mb-1"
                    >SSH Key Path</label
                  >
                  <input
                    id="id_connection_ssh_key_path"
                    type="text"
                    bind:value={currentConnection.ssh_key_path}
                    class="w-full bg-gray-950 border border-gray-700 rounded px-3 py-2 focus:border-blue-500 focus:outline-none"
                    placeholder="~/.ssh/id_rsa"
                    autocapitalize="off"
                    autocomplete="off"
                    spellcheck="false"
                    autocorrect="off"
                  />
                </div>
              </div>
            </div>
          {/if}
        </div>

        {#if error}
          <div
            class="text-red-500 text-sm p-3 bg-red-900/20 rounded border border-red-900/50"
          >
            {error}
          </div>
        {/if}

        <div class="flex gap-4 pt-4">
          <button
            onclick={connect}
            disabled={loading}
            class="flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-4 rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
          >
            {#if loading}
              Connecting...
            {:else}
              <Plug size={18} /> Connect
            {/if}
          </button>
          <button
            onclick={save}
            class="px-4 py-2 bg-gray-800 hover:bg-gray-700 text-white font-medium rounded transition-colors flex items-center gap-2"
          >
            <Save size={18} /> Save
          </button>
        </div>
      </div>
    </div>
  </div>
</div>
