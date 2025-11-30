import {
  connectDb,
  listDatabases,
  listTables,
  type SavedConnection,
} from "$lib/db";

type WorkSpaceStatus = "initial" | "connecting" | "connected" | "error";

const tunnels = new Map();

export class WorkSpace {
  #id: string;
  #config: SavedConnection;

  #databases: string[] = $state<string[]>([]);
  #tables: string[] = $state<string[]>([]);
  #currentDatabase: string = $state<string>("");
  #activeTabId: string | null = $state<string | null>(null);
  #status = $state<WorkSpaceStatus>("initial");

  #changed: () => void | Promise<void>;

  constructor(
    data: {
      id: string;
      config: SavedConnection;
      databases: string[];
      tables: string[];
      currentDatabase: string;
      activeTabId?: string; // Track the active tab for this connection
    },
    changed: () => void | Promise<void>
  ) {
    this.#id = data.id;
    this.#config = data.config;
    this.#databases = data.databases;
    this.#tables = data.tables;
    this.#currentDatabase = data.currentDatabase;

    if (data.activeTabId) {
      this.#activeTabId = data.activeTabId;
    }

    this.#changed = changed;
  }

  async connect() {
    if (this.#status === "connecting" || this.#status === "connected") {
      return;
    }

    this.#status = "connecting";

    try {
      const tunnelKey = getTunnelKey(this.#config);
      if (this.#config.ssh_enabled && !tunnels.has(tunnelKey)) {
        await tunnels.get(tunnelKey);
      }

      const promise = connectDb(this.#id, this.#config);

      if (this.#config.ssh_enabled) {
        tunnels.set(tunnelKey, promise);
      }

      await promise;

      if (this.#config.ssh_enabled) {
        tunnels.delete(tunnelKey);
      }

      const [databases, tables] = await Promise.all([
        listDatabases(this.#id),
        listTables(this.#id),
      ]);

      this.#databases = databases;
      this.#tables = tables;

      this.#status = "connected";
    } catch (error) {
      this.#status = "error";
      console.error("Failed to connect:", error);
    }
  }

  async disconnect() {}

  get key() {
    return getSpaceKey(this.#config, this.#currentDatabase);
  }

  get id() {
    return this.#id;
  }

  get name() {
    return this.#config.name;
  }

  get config() {
    return this.#config;
  }

  get databases() {
    return this.#databases;
  }

  set databases(value: string[]) {
    this.#databases = value;
    this.#changed();
  }

  get tables() {
    return this.#tables;
  }

  set tables(value: string[]) {
    this.#tables = value;
    this.#changed();
  }

  get currentDatabase() {
    return this.#currentDatabase;
  }

  set currentDatabase(value: string) {
    this.#currentDatabase = value;
    this.#changed();
  }

  get activeTabId() {
    return this.#activeTabId;
  }

  set activeTabId(value: string | null) {
    this.#activeTabId = value;
    this.#changed();
  }

  get status() {
    return this.#status;
  }
}

export function getTunnelKey(config: SavedConnection): string {
  if (!config.ssh_enabled) {
    return "";
  }

  return config.ssh_host + "::" + config.ssh_port + "::" + config.ssh_user;
}

export function getSpaceKey(config: SavedConnection, current: string): string {
  const tunnelKey = getTunnelKey(config);
  return `${tunnelKey}::${config.host}::${config.port}::${config.username}::${current}`;
}
