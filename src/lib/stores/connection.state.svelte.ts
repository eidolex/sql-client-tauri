import {
  deleteConnection,
  loadConnections,
  saveConnection,
  type SavedConnection,
} from "$lib/db";
import { ask } from "@tauri-apps/plugin-dialog";
import { getContext, setContext } from "svelte";

export class ConnectionState {
  connections = $state<SavedConnection[]>([]);
  selectedConnectionId = $state<string | null>(null);

  async loadSavedConnections() {
    try {
      this.connections = await loadConnections();
    } catch (e) {
      console.error("Failed to load connections", e);
    }
  }

  async remove(id: string) {
    const confirmed = await ask(
      "Are you sure you want to delete this connection?",
      {
        title: "Delete Connection",
        kind: "warning",
      }
    );
    if (!confirmed) return;

    try {
      await deleteConnection(id);
      await this.loadSavedConnections();
    } catch (e: any) {
      // TODO: fix message display
      //   error = e.message || "Failed to delete connection";
    }
  }

  async save(connection: SavedConnection) {
    try {
      await saveConnection(connection);
      await this.loadSavedConnections();
    } catch (e: any) {
      throw new Error(e.message || "Failed to save connection");
    }
  }
}

const CONNECTION_STATE_KEY = "$_connection_state";

export const getConnectionState = () => {
  return getContext<ConnectionState>(CONNECTION_STATE_KEY);
};

export const setConnectionState = () => {
  const state = new ConnectionState();
  return setContext<ConnectionState>(CONNECTION_STATE_KEY, state);
};
