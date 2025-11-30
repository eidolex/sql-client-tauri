import {
  loadAppState,
  saveAppState,
  type AppStateData,
  type SavedConnection,
} from "../db";
import { SvelteMap } from "svelte/reactivity";
import { getSpaceKey, WorkSpace } from "./work-space.state.svelte";
import {
  TableTab,
  TableDataTab,
  TableQueryTab,
  type ITableDataTab,
  type TableTabOptions,
} from "./table-tab.state.svelte";

export class AppState {
  #spaces = new SvelteMap<string, WorkSpace>();
  #tabs = $state<TableTab<any>[]>([]);
  #selectedSpaceId = $state<string | null>(null);

  isFirstNavigation = true;
  #isSaving = false;
  #saveTimeout: number | null = null;

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

    this.#tabs.push(new TableTab(tab, this.saveState.bind(this)));

    space.activeTabId = tab.id;

    this.saveState();

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

    this.#tabs.splice(index, 1);

    if (previousTab) {
      this.#spaces.get(previousTab.connectionId)!.activeTabId = previousTab.id;
    } else {
      const nextTab = this.#tabs.find(
        (t, i) => t.connectionId === tab.connectionId && i >= index
      );
      if (nextTab) {
        this.#spaces.get(nextTab.connectionId)!.activeTabId = nextTab.id;
      }
    }

    this.saveState();
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

    const newSpace = new WorkSpace(
      {
        id: connectionId,
        config: connection,
        databases: [],
        tables: [],
        currentDatabase: connection.database,
      },
      this.saveState.bind(this)
    );

    this.#spaces.set(connectionId, newSpace);
    this.saveState();

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
    this.#tabs = this.#tabs.filter((t) => t.connectionId !== connectionId);

    this.saveState();
    // this.tabs.delete(connectionId);
    return prev;
  }

  async loadState() {
    try {
      this.#isSaving = true;
      const state = await loadAppState();

      console.log("Loaded state", state);

      if (state) {
        this.selectedSpaceId = state.selected_space_id ?? null;
        state.spaces.forEach((space) => {
          this.#spaces.set(
            space.id,
            new WorkSpace(
              {
                id: space.id,
                config: space.config,
                databases: [],
                tables: [],
                currentDatabase: space.current_database,
                activeTabId: space.active_tab_id ?? undefined,
              },
              this.saveState.bind(this)
            )
          );
        });
        this.#tabs = state.tabs.map(
          (tab) =>
            new TableTab(
              {
                id: tab.id,
                title: tab.title,
                connectionId: tab.connection_id,
                database: tab.database,
                type: tab.type,
                data: tab.data,
              } as TableTabOptions,
              this.saveState.bind(this)
            )
        );
      }
    } catch (e) {
      console.error("Failed to load state", e);
    } finally {
      this.#isSaving = false;
    }
  }

  async saveState() {
    if (this.#isSaving) {
      return;
    }

    if (this.#saveTimeout) {
      clearTimeout(this.#saveTimeout);
    }

    this.#saveTimeout = setTimeout(() => {
      this.#actuallySaveState();
    }, 1000);
  }

  async #actuallySaveState() {
    this.#isSaving = true;

    const state: AppStateData = {
      selected_space_id: this.selectedSpaceId ?? undefined,
      spaces: Array.from(this.#spaces.values()).map((space) => ({
        id: space.id,
        config: space.config,
        current_database: space.currentDatabase,
        active_tab_id: space.activeTabId ?? undefined,
      })),
      tabs: this.#tabs.map((tab) => {
        const baseTab = {
          id: tab.id,
          title: tab.title,
          connection_id: tab.connectionId,
          database: tab.database,
          type: tab.type,
        };

        if (tab.type === "query") {
          const queryData = tab.data as TableQueryTab;
          return {
            ...baseTab,
            data: {
              query: queryData.query,
            },
          };
        } else {
          const tableData = tab.data as TableDataTab;
          return {
            ...baseTab,
            data: {
              table: tableData.table,
              page: tableData.page,
              pageSize: tableData.pageSize,
            },
          };
        }
      }),
    };

    try {
      console.log("Saving state", state);
      await saveAppState(state);
    } catch (e) {
      console.error("Failed to save state", e);
    } finally {
      this.#isSaving = false;
      this.#saveTimeout = null;
    }
  }

  get spaces() {
    return this.#spaces;
  }

  get tabs() {
    return this.#tabs;
  }

  get selectedSpaceId() {
    return this.#selectedSpaceId;
  }

  set selectedSpaceId(id: string | null) {
    this.#selectedSpaceId = id;
    this.saveState();
  }
}

let appState: AppState = new AppState();

export const getAppState = () => {
  return appState;
};
