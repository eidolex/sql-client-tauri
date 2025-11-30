import { type SavedConnection } from "../db";
import { SvelteMap } from "svelte/reactivity";
import { getSpaceKey, WorkSpace } from "./work-space.state.svelte";
import {
  TableTab,
  type ITableDataTab,
  type TableTabOptions,
} from "./table-tab.state.svelte";

export class AppState {
  #spaces = new SvelteMap<string, WorkSpace>();
  #tabs = $state<TableTab<any>[]>([]);

  addTab(tab: TableTabOptions) {
    const space = this.#spaces.get(tab.connectionId);

    if (!space) {
      return;
    }

    const existing = this.#tabs.find((t) => {
      if (t.type == "query" && tab.type === t.type) {
        return true;
      }

      return (
        t.connectionId == tab.connectionId &&
        (t as ITableDataTab).data.table === (tab as ITableDataTab).data.table
      );
    });

    if (existing) {
      space.activeTabId = existing.id;
      return existing.id;
    }

    this.#tabs.push(new TableTab(tab));

    space.activeTabId = tab.id;

    return tab.id;
  }

  closeTab(tabId: string) {
    const index = this.#tabs.findIndex((t) => t.id === tabId);

    if (index === -1) {
      return;
    }

    const tab = this.#tabs[index];

    const previousTab = this.#tabs.findLast(
      (t, i) => t.connectionId === tab.connectionId && i < index
    );

    this.#tabs.splice(index);

    if (previousTab) {
      this.#spaces.get(previousTab.connectionId)!.activeTabId = previousTab.id;
    }
  }

  closeActiveTab(spaceId: string) {
    const activeTabId = this.#spaces.get(spaceId)?.activeTabId;
    if (activeTabId) {
      this.closeTab(activeTabId);
    }
  }

  hasConnection(connection: SavedConnection) {
    const newKey = getSpaceKey(connection, connection.database);

    for (const space of this.#spaces.values()) {
      if (space.key === newKey) {
        return true;
      }
    }

    return false;
  }

  getConnectionId(connection: SavedConnection) {
    const newKey = getSpaceKey(connection, connection.database);

    for (const space of this.#spaces.values()) {
      if (space.key === newKey) {
        return space.id;
      }
    }

    if (!this.#spaces.has(connection.id)) {
      return connection.id;
    }

    return crypto.randomUUID();
  }

  async addSpace(connection: SavedConnection, connect: boolean = false) {
    const connectionId = this.getConnectionId(connection);

    const space = this.#spaces.get(connectionId);

    if (space) {
      return space.id;
    }

    const newSpace = new WorkSpace({
      id: connectionId,
      config: connection,
      databases: [],
      tables: [],
      currentDatabase: connection.database,
    });

    this.#spaces.set(connectionId, newSpace);

    if (!connect) {
      return connectionId;
    }

    await newSpace.connect();

    return connectionId;
  }

  removeSpace(connectionId: string) {
    const keys = this.#spaces.keys();

    let prev: string | null = null;

    for (const key of keys) {
      if (key === connectionId) {
        break;
      }
      prev = key;
    }

    this.#spaces.delete(connectionId);

    // this.tabs.delete(connectionId);
    return prev;
  }

  get spaces() {
    return this.#spaces;
  }

  get tabs() {
    return this.#tabs;
  }
}

let appState: AppState = new AppState();

export const getAppState = () => {
  return appState;
};
