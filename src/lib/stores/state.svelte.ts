import { getContext, setContext } from "svelte";
import {
  connectDb,
  listDatabases,
  listTables,
  type ColumnDefinition,
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

type BaseTab = {
  id: string;
  title: string;
  connectionId: string;
  database: string;
};

export type DataTab = BaseTab & {
  type: "data";
  table: string;
  columns?: ColumnDefinition[];
  page: number;
  pageSize: number;
  totalRows: number;
  data?: any[];
};

export type StructureTab = BaseTab & {
  type: "structure";
  table: string;
  columns?: ColumnDefinition[];
  page?: number;
  pageSize?: number;
  totalRows?: number;
  data?: any[];
};

export type QueryTab = BaseTab & {
  type: "query";
  query?: string;
};

export type Tab = DataTab | StructureTab | QueryTab;
export class AppState {
  spaces = new SvelteMap<string, ActiveConnection>();
  tunnels = new SvelteMap<string, Promise<unknown>>();
  tabs = new SvelteMap<string, Tab[]>();
  // This now tracks the globally selected connection (top level tab)
  selectedConnectionId = $state<string | null>(null);

  // We can keep this for backward compatibility or easy access,
  // but the source of truth for "what is visible" depends on selectedConnectionId
  // If selectedConnectionId is null, we are in "Home" or "New Connection" view
  // If selectedConnectionId is set, we show tabs for that connection

  // Helper to get the active tab ID for the current connection
  // get activeTabId() {
  //   if (!this.selectedConnectionId) return null;
  //   const conn = this.activeConnections.find(
  //     (c) => c.id === this.selectedConnectionId
  //   );
  //   return conn?.activeTabId ?? null;
  // }

  // set activeTabId(id: string | null) {
  //   if (this.selectedConnectionId) {
  //     const conn = this.activeConnections.find(
  //       (c) => c.id === this.selectedConnectionId
  //     );
  //     if (conn && id) {
  //       conn.activeTabId = id;
  //     }
  //   }
  // }

  constructor() {}

  addTab(connectionId: string, tab: Tab) {
    if (!this.tabs.has(connectionId)) {
      this.tabs.set(connectionId, [tab]);
      this.selectTab(connectionId, tab.id);
      return;
    }
    const tabs = this.tabs.get(connectionId)?.slice() || [];

    tabs.push(tab);

    this.tabs.set(connectionId, tabs);
    this.selectTab(connectionId, tab.id);
  }

  closeTab(connectionId: string, tabId: string) {
    if (!this.tabs.has(connectionId)) {
      return;
    }
    const tabs = this.tabs.get(connectionId) || [];
    const newTabs = tabs.filter((t) => t.id !== tabId);
    this.tabs.set(connectionId, newTabs);

    const lastTab = newTabs.at(-1);

    if (lastTab) {
      this.selectTab(connectionId, lastTab.id);
    }
  }

  selectTab(connectionId: string, tabId: string) {
    if (!this.tabs.has(connectionId)) {
      return;
    }
    const space = this.spaces.get(connectionId);

    if (!space) {
      return;
    }

    this.spaces.set(connectionId, { ...space, activeTabId: tabId });
  }

  updateTab(connectionId: string, tab: Tab) {
    if (!this.tabs.has(connectionId)) {
      return;
    }
    const tabs = this.tabs.get(connectionId) || [];
    this.tabs.set(
      connectionId,
      tabs.map((t) => (t.id === tab.id ? tab : t))
    );
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
    this.tabs.delete(connectionId);
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
