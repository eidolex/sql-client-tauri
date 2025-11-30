import { invoke } from "@tauri-apps/api/core";

export interface DatabaseError {
    message: string;
}

export interface SavedConnection {
    id: string;
    name: string;
    host: string;
    port: number;
    username: string;
    password?: string;
    database: string;
    db_type: 'postgres' | 'mysql';
    ssh_enabled: boolean;
    ssh_host?: string;
    ssh_port?: number;
    ssh_user?: string;
    ssh_password?: string;
    ssh_key_path?: string;
}

export async function connectDb(connectionId: string, connectionConfig: SavedConnection): Promise<void> {
    return await invoke("connect_db", { connectionId, connectionConfig });
}

export async function disconnectDb(connectionId: string): Promise<void> {
    return await invoke("disconnect_db", { connectionId });
}

export async function listDatabases(connectionId: string): Promise<string[]> {
    return await invoke("list_databases", { connectionId });
}

export async function listTables(connectionId: string): Promise<string[]> {
    return await invoke("list_tables", { connectionId });
}

export interface QueryResult {
    columns: string[];
    rows: any[][];
    total_rows?: number;
}

export interface Filter {
    field: string;
    operator: string;
    value: string;
}

export interface Sort {
    field: string;
    order: "ASC" | "DESC";
}

export async function getTableData(connectionId: string, tableName: string, limit: number, offset: number, filters: Filter[] = [], sorts: Sort[] = []): Promise<QueryResult> {
    return await invoke("get_table_data", { connectionId, tableName, limit, offset, filters, sorts });
}

export interface ColumnDefinition {
    column_name: string;
    data_type: string;
    is_nullable: string;
    column_default?: string;
    comment?: string;
    foreign_key?: string;
}

export interface IndexDefinition {
    index_name: string;
    index_algorithm: string;
    is_unique: boolean;
    is_primary: boolean;
    column_names: string;
    condition?: string;
    comment?: string;
}

export async function getTableStructure(connectionId: string, tableName: string): Promise<ColumnDefinition[]> {
    return await invoke("get_table_structure", { connectionId, tableName });
}

export async function getTableIndexes(connectionId: string, tableName: string): Promise<IndexDefinition[]> {
    return await invoke("get_table_indexes", { connectionId, tableName });
}

export async function executeQuery(connectionId: string, query: string): Promise<QueryResult> {
    return await invoke("execute_query", { connectionId, query });
}

export async function getDatabaseSchema(connectionId: string): Promise<Record<string, string[]>> {
    return await invoke("get_database_schema", { connectionId });
}

export async function saveConnection(connection: SavedConnection): Promise<void> {
    return await invoke("save_connection", { connection });
}

export async function loadConnections(): Promise<SavedConnection[]> {
    return await invoke("load_connections");
}

export async function deleteConnection(id: string): Promise<void> {
    return await invoke("delete_connection", { id });
}

// State persistence types
export interface TabState {
    type: "data" | "structure" | "query";
    id: string;
    title: string;
    connection_id: string;
    database: string;
    table?: string;
    page?: number;
    page_size?: number;
    query?: string;
}

export interface SpaceState {
    id: string;
    config_id: string;
    current_database: string;
    active_tab_id?: string;
    tabs: TabState[];
}

export interface AppStateData {
    spaces: SpaceState[];
    selected_connection_id?: string;
}

export async function saveAppState(state: AppStateData): Promise<void> {
    return await invoke("save_app_state", { state });
}

export async function loadAppState(): Promise<AppStateData | null> {
    return await invoke("load_app_state");
}
