import { getContext, setContext } from "svelte";
import type { SavedConnection } from "./db";

export interface ActiveConnection {
  id: string; // The runtime connection ID returned by backend
  config: SavedConnection;
  databases: string[];
  tables: string[];
  currentDatabase: string;
  activeTabId?: string; // Track the active tab for this connection
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

  getConnection(connectionId: string) {
    return this.activeConnections.find((c) => c.id === connectionId);
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
}

const APP_STATE_KEY = "$_app_state";

export const getAppState = () => {
  return getContext<AppState>(APP_STATE_KEY);
};

export const setAppState = () => {
  const state = new AppState();
  return setContext<AppState>(APP_STATE_KEY, state);
};
