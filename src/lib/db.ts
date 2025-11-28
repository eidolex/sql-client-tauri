import { invoke } from "@tauri-apps/api/core";

export interface DatabaseError {
    message: string;
}

export async function connectDb(connectionString: string): Promise<string> {
    return await invoke("connect_db", { connectionString });
}

export async function listDatabases(): Promise<string[]> {
    return await invoke("list_databases");
}

export async function listTables(): Promise<string[]> {
    return await invoke("list_tables");
}

export interface QueryResult {
    columns: string[];
    rows: any[][];
}

export async function getTableData(tableName: string): Promise<QueryResult> {
    return await invoke("get_table_data", { tableName });
}

export async function getTableStructure(tableName: string): Promise<any[]> {
    return await invoke("get_table_structure", { tableName });
}

export async function executeQuery(query: string): Promise<QueryResult> {
    return await invoke("execute_query", { query });
}

export async function getDatabaseSchema(): Promise<Record<string, string[]>> {
    return await invoke("get_database_schema");
}
