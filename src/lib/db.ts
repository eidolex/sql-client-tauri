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
    ssh_enabled: boolean;
    ssh_host?: string;
    ssh_port?: number;
    ssh_user?: string;
    ssh_password?: string;
    ssh_key_path?: string;
}

export async function connectDb(connectionConfig: SavedConnection): Promise<string> {
    return await invoke("connect_db", { connectionConfig });
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

export async function getTableStructure(connectionId: string, tableName: string): Promise<any[]> {
    return await invoke("get_table_structure", { connectionId, tableName });
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
