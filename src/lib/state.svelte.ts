export class AppState {
    isConnected = $state(false);
    databases = $state<string[]>([]);
    tables = $state<string[]>([]);
    currentDatabase = $state("");
    currentTable = $state("");
    currentView = $state<"data" | "structure" | "query">("data");

    constructor() { }
}

export const appState = new AppState();
