import { getContext, setContext } from "svelte";
import {
  connectDb,
  listDatabases,
  listTables,
  type SavedConnection,
} from "../db";
import { SvelteMap } from "svelte/reactivity";

export interface ActiveConnection {
  id: string; // The runtime connection ID returned by backend
  config: SavedConnection;
  databases: string[];
  tables: string[];
  currentDatabase: string;
  activeTabId?: string; // Track the active tab for this connection
  status: "initial" | "connecting" | "connected" | "error";
}

export interface Tab {
  id: string;
  title: string;
  type: "data" | "structure" | "query";
  connectionId: string;
  database: string;
  table?: string;
  query?: string;
  // View State
  page?: number;
  pageSize?: number;
  data?: any[];
  columns?: string[];
  totalRows?: number;
}

export class AppState {
  spaces = new SvelteMap<string, ActiveConnection>();
  tunnels = new SvelteMap<string, Promise<unknown>>();

  activeConnections = $state<ActiveConnection[]>([]);
  tabs = $state<Tab[]>([]);
  // This now tracks the globally selected connection (top level tab)
  selectedConnectionId = $state<string | null>(null);

  // We can keep this for backward compatibility or easy access,
  // but the source of truth for "what is visible" depends on selectedConnectionId
  // If selectedConnectionId is null, we are in "Home" or "New Connection" view
  // If selectedConnectionId is set, we show tabs for that connection

  // Helper to get the active tab ID for the current connection
  get activeTabId() {
    if (!this.selectedConnectionId) return null;
    const conn = this.activeConnections.find(
      (c) => c.id === this.selectedConnectionId
    );
    return conn?.activeTabId ?? null;
  }

  set activeTabId(id: string | null) {
    if (this.selectedConnectionId) {
      const conn = this.activeConnections.find(
        (c) => c.id === this.selectedConnectionId
      );
      if (conn && id) {
        conn.activeTabId = id;
      }
    }
  }

  constructor() {}

  addConnection(connection: ActiveConnection) {
    this.activeConnections.push(connection);
    this.selectedConnectionId = connection.id;
  }

  removeConnection(connectionId: string) {
    this.activeConnections = this.activeConnections.filter(
      (c) => c.id !== connectionId
    );
    // Close tabs associated with this connection
    this.tabs = this.tabs.filter((t) => t.connectionId !== connectionId);

    if (this.selectedConnectionId === connectionId) {
      this.selectedConnectionId =
        this.activeConnections.length > 0
          ? this.activeConnections[this.activeConnections.length - 1].id
          : null;
    }
  }

  addTab(tab: Tab) {
    this.tabs.push(tab);
    // When adding a tab, make sure we switch to that connection and set it as active
    this.selectedConnectionId = tab.connectionId;
    const conn = this.activeConnections.find((c) => c.id === tab.connectionId);
    if (conn) {
      conn.activeTabId = tab.id;
    }
  }

  closeTab(tabId: string) {
    const index = this.tabs.findIndex((t) => t.id === tabId);
    if (index !== -1) {
      const tab = this.tabs[index];
      const connectionId = tab.connectionId;

      this.tabs.splice(index, 1);

      // If this was the active tab for its connection, switch to another one
      const conn = this.activeConnections.find((c) => c.id === connectionId);
      if (conn && conn.activeTabId === tabId) {
        const remainingTabs = this.tabs.filter(
          (t) => t.connectionId === connectionId
        );
        conn.activeTabId =
          remainingTabs.length > 0
            ? remainingTabs[remainingTabs.length - 1].id
            : undefined;
      }
    }
  }

  updateConnectionId(
    oldId: string,
    newId: string,
    newDatabase: string,
    newTables: string[]
  ) {
    const conn = this.activeConnections.find((c) => c.id === oldId);
    if (conn) {
      conn.id = newId;
      conn.currentDatabase = newDatabase;
      conn.tables = newTables;
      conn.config.database = newDatabase;
    }

    if (this.selectedConnectionId === oldId) {
      this.selectedConnectionId = newId;
    }

    // Update tabs
    this.tabs.forEach((tab) => {
      if (tab.connectionId === oldId) {
        tab.connectionId = newId;
        tab.database = newDatabase;
        // We might want to clear data/table if the table doesn't exist in new DB
        // But for now let's keep it and let the user refresh/fail
      }
    });
  }

  hasConnection(connection: SavedConnection) {
    const tunnelKey = this.generateTunnelKey(connection);
    const newKey = `${tunnelKey}::${connection.host}::${connection.port}::${connection.username}::${connection.database}`;

    for (const space of this.spaces.values()) {
      const spaceTunnelKey = this.generateTunnelKey(space.config);
      const spaceKey = `${spaceTunnelKey}::${space.config.host}::${space.config.port}::${space.config.username}::${space.currentDatabase}`;
      if (spaceKey === newKey) {
        return true;
      }
    }

    return false;
  }

  getConnectionId(connection: SavedConnection) {
    if (this.hasConnection(connection)) {
      return connection.id;
    }

    if (!this.spaces.has(connection.id)) {
      return connection.id;
    }

    return crypto.randomUUID();
  }

  async addSpace(connection: SavedConnection, connect: boolean = false) {
    const connectionId = this.getConnectionId(connection);

    if (this.spaces.has(connectionId)) {
      return connectionId;
    }

    this.spaces.set(connectionId, {
      id: connectionId,
      config: connection,
      databases: [],
      tables: [],
      currentDatabase: connection.database,
      status: "initial",
    });

    if (!connect) {
      return connectionId;
    }

    await this.connectSpace(connectionId);

    return connectionId;
  }

  removeSpace(connectionId: string) {
    this.spaces.delete(connectionId);
    return this.spaces.keys().toArray().at(-1) ?? null;
  }

  async connectSpace(connectionId: string) {
    const space = this.spaces.get(connectionId);

    if (!space) {
      throw new Error("Space not found");
    }

    if (space.status === "connecting" || space.status === "connected") {
      return;
    }

    const connection = space.config;

    try {
      if (connection.ssh_enabled) {
        const key = this.generateTunnelKey(connection);
        if (this.tunnels.has(key)) {
          // Reuse existing tunnel promise
          await this.tunnels.get(key);
        }
      }

      const promise = connectDb(connectionId, connection);

      if (connection.ssh_enabled) {
        const key = this.generateTunnelKey(connection);
        this.tunnels.set(key, promise);
      }

      this.spaces.set(connectionId, {
        ...space,
        status: "connecting",
      });

      await promise;

      if (connection.ssh_enabled) {
        this.tunnels.delete(this.generateTunnelKey(connection));
      }

      const [databases, tables] = await Promise.all([
        listDatabases(connectionId),
        listTables(connectionId),
      ]);

      this.spaces.set(connectionId, {
        ...space,
        databases: databases,
        tables: tables,
        status: "connected",
      });
    } catch (error) {
      this.spaces.set(connectionId, {
        ...space,
        status: "error",
      });
    }
  }

  private generateTunnelKey(connection: SavedConnection): string {
    if (!connection.ssh_enabled) {
      return "";
    }

    return (
      connection.ssh_host +
      "::" +
      connection.ssh_port +
      "::" +
      connection.ssh_user
    );
  }
}

const APP_STATE_KEY = "$_app_state";

export const getAppState = () => {
  return getContext<AppState>(APP_STATE_KEY);
};

export const setAppState = () => {
  const state = new AppState();
  return setContext<AppState>(APP_STATE_KEY, state);
};
