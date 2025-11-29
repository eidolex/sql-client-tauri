import { getContext, setContext } from "svelte";
import {
  connectDb,
  listDatabases,
  listTables,
  type SavedConnection,
} from "../db";
import { SvelteMap } from "svelte/reactivity";
import { TableTabState, type Tab } from "./table-tab.state.svelte";

export interface ActiveConnection {
  id: string; // The runtime connection ID returned by backend
  config: SavedConnection;
  databases: string[];
  tables: string[];
  currentDatabase: string;
  activeTabId?: string; // Track the active tab for this connection
  status: "initial" | "connecting" | "connected" | "error";
}
export class AppState {
  spaces = new SvelteMap<string, ActiveConnection>();
  tunnels = new SvelteMap<string, Promise<unknown>>();
  tabs = new SvelteMap<string, TableTabState>();
  // This now tracks the globally selected connection (top level tab)
  selectedConnectionId = $state<string | null>(null);

  constructor() {}

  addTab(connectionId: string, tab: Tab) {
    if (!this.tabs.has(connectionId)) {
      this.tabs.set(connectionId, new TableTabState());
    }
    this.tabs.get(connectionId)?.addTab(tab);
    this.selectTab(connectionId, tab.id);
  }

  closeTab(connectionId: string, tabId: string) {
    if (!this.tabs.has(connectionId)) {
      return;
    }
    const tabState = this.tabs.get(connectionId);

    tabState?.removeTab(tabId);

    const lastTab = tabState?.items.at(-1);

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

  // updateTab(connectionId: string, tab: Tab) {
  //   if (!this.tabs.has(connectionId)) {
  //     return;
  //   }
  //   const tabs = this.tabs.get(connectionId) || [];
  //   this.tabs.set(
  //     connectionId,
  //     tabs.map((t) => (t.id === tab.id ? tab : t))
  //   );
  // }

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
