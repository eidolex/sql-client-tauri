import type { SavedConnection } from "./db";

export interface ActiveConnection {
    id: string; // The runtime connection ID returned by backend
    config: SavedConnection;
    databases: string[];
    tables: string[];
    currentDatabase: string;
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
    activeTabId = $state<string | null>(null);

    constructor() { }

    addConnection(connection: ActiveConnection) {
        this.activeConnections.push(connection);
    }

    removeConnection(connectionId: string) {
        this.activeConnections = this.activeConnections.filter(c => c.id !== connectionId);
        // Close tabs associated with this connection
        this.tabs = this.tabs.filter(t => t.connectionId !== connectionId);
        if (this.activeTabId && !this.tabs.find(t => t.id === this.activeTabId)) {
            this.activeTabId = this.tabs.length > 0 ? this.tabs[this.tabs.length - 1].id : null;
        }
    }

    addTab(tab: Tab) {
        this.tabs.push(tab);
        this.activeTabId = tab.id;
    }

    closeTab(tabId: string) {
        const index = this.tabs.findIndex(t => t.id === tabId);
        if (index !== -1) {
            this.tabs.splice(index, 1);
            if (this.activeTabId === tabId) {
                this.activeTabId = this.tabs.length > 0
                    ? this.tabs[Math.min(index, this.tabs.length - 1)].id
                    : null;
            }
        }
    }

    getConnection(connectionId: string) {
        return this.activeConnections.find(c => c.id === connectionId);
    }

    updateConnectionId(oldId: string, newId: string, newDatabase: string, newTables: string[]) {
        const conn = this.activeConnections.find(c => c.id === oldId);
        if (conn) {
            conn.id = newId;
            conn.currentDatabase = newDatabase;
            conn.tables = newTables;
            conn.config.database = newDatabase;
        }

        // Update tabs
        this.tabs.forEach(tab => {
            if (tab.connectionId === oldId) {
                tab.connectionId = newId;
                tab.database = newDatabase;
                // We might want to clear data/table if the table doesn't exist in new DB
                // But for now let's keep it and let the user refresh/fail
            }
        });
    }
}

export const appState = new AppState();
