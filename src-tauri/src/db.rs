use crate::connection_manager::SavedConnection;
use crate::ssh_tunnel::{SshTunnel, TunnelConfig};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Pool, Postgres, Row, TypeInfo};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use tauri::State;

pub struct DbConnection {
    pub pool: Pool<Postgres>,
    pub ssh_tunnel: Option<Arc<SshTunnel>>,
}

pub struct AppState {
    pub connections: Mutex<HashMap<String, DbConnection>>,
    pub tunnels: Mutex<HashMap<TunnelConfig, Weak<SshTunnel>>>,
}

#[derive(serde::Serialize)]
pub struct DatabaseError {
    message: String,
}

impl From<sqlx::Error> for DatabaseError {
    fn from(err: sqlx::Error) -> Self {
        DatabaseError {
            message: err.to_string(),
        }
    }
}

// Helper to get pool
fn get_pool(
    state: &State<'_, AppState>,
    connection_id: &str,
) -> Result<Pool<Postgres>, DatabaseError> {
    let connections = state.connections.lock().unwrap();
    let conn = connections.get(connection_id).ok_or(DatabaseError {
        message: "Connection not found".to_string(),
    })?;
    Ok(conn.pool.clone())
}

#[tauri::command]
pub async fn connect_db(
    connection_config: SavedConnection,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    // 1. Handle SSH Tunnel if enabled
    let mut db_host = connection_config.host.clone();
    let mut db_port = connection_config.port;
    let mut ssh_tunnel = None;

    if connection_config.ssh_enabled {
        if let Some(ssh_host) = &connection_config.ssh_host {
            let config = TunnelConfig {
                ssh_host: ssh_host.clone(),
                ssh_port: connection_config.ssh_port,
                ssh_user: connection_config.ssh_user.clone().filter(|s| !s.is_empty()),
                ssh_password: connection_config
                    .ssh_password
                    .clone()
                    .filter(|s| !s.is_empty()),
                ssh_key_path: connection_config
                    .ssh_key_path
                    .clone()
                    .filter(|s| !s.is_empty()),
                remote_host: db_host.clone(),
                remote_port: db_port,
            };

            // Check for existing tunnel
            let mut tunnels = state.tunnels.lock().unwrap();
            if let Some(weak_tunnel) = tunnels.get(&config) {
                if let Some(existing_tunnel) = weak_tunnel.upgrade() {
                    eprintln!(
                        "Reusing existing SSH tunnel on port {}",
                        existing_tunnel.get_local_port()
                    );
                    db_host = "127.0.0.1".to_string();
                    db_port = existing_tunnel.get_local_port();
                    ssh_tunnel = Some(existing_tunnel);
                }
            }

            if ssh_tunnel.is_none() {
                eprintln!("Starting new SSH tunnel...");
                let (tunnel, actual_local_port) =
                    SshTunnel::start(config.clone()).map_err(|e| DatabaseError { message: e })?;

                let tunnel_arc = Arc::new(tunnel);
                tunnels.insert(config, Arc::downgrade(&tunnel_arc));

                ssh_tunnel = Some(tunnel_arc);
                db_host = "127.0.0.1".to_string();
                db_port = actual_local_port;
            }
        }
    }

    // 2. Construct Connection String
    let password = connection_config.password.as_deref().unwrap_or("");
    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        connection_config.username, password, db_host, db_port, connection_config.database
    );

    // 3. Connect to Database
    let pool_result = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await;

    let pool = match pool_result {
        Ok(pool) => pool,
        Err(e) => {
            // Clean up SSH process if connection fails
            // If we created a new tunnel (ssh_tunnel is Some), dropping it here will stop it
            // if it's the only reference.
            // ssh_tunnel is Option<Arc<SshTunnel>>.
            // If we inserted it into the map, the map has a Weak reference.
            // If we drop this Arc, the strong count goes to 0, and it is dropped.
            // The Weak reference in the map will then fail to upgrade.
            // So we don't need to manually stop it.
            return Err(DatabaseError::from(e));
        }
    };

    let connection_id = uuid::Uuid::new_v4().to_string();
    let connection = DbConnection { pool, ssh_tunnel };

    state
        .connections
        .lock()
        .unwrap()
        .insert(connection_id.clone(), connection);

    Ok(connection_id)
}

#[tauri::command]
pub async fn disconnect_db(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<(), DatabaseError> {
    let conn = {
        let mut connections = state.connections.lock().unwrap();
        connections.remove(&connection_id)
    };

    if let Some(conn) = conn {
        conn.pool.close().await;
        // Tunnel is automatically stopped when the last Arc is dropped
        // We don't need to manually call stop() anymore, or we can if we want to force it,
        // but since we are sharing it, we should NOT force stop it unless we are sure.
        // The Drop implementation of SshTunnel calls stop().
        // When we remove the connection from the map, the DbConnection is dropped.
        // This drops the Arc<SshTunnel>. If it's the last one, SshTunnel is dropped and stopped.
    }
    Ok(())
}

#[tauri::command]
pub async fn list_databases(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false;")
        .fetch_all(&pool)
        .await?;

    let databases: Vec<String> = rows.iter().map(|row| row.get("datname")).collect();
    Ok(databases)
}

#[tauri::command]
pub async fn list_tables(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    let rows = sqlx::query(
        "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public';",
    )
    .fetch_all(&pool)
    .await?;

    let tables: Vec<String> = rows
        .iter()
        .map(|row: &PgRow| row.get("table_name"))
        .collect();
    Ok(tables)
}

#[derive(serde::Serialize)]
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    total_rows: Option<i64>,
}

fn row_to_values(row: PgRow) -> Vec<serde_json::Value> {
    let mut values = Vec::new();
    for col in row.columns() {
        let col_name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name();

        let value = match type_name {
            "BOOL" => {
                let val: Option<bool> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT2" | "INT4" | "INT" => {
                let val: Option<i32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT8" | "BIGINT" => {
                let val: Option<i64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT4" | "REAL" => {
                let val: Option<f32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let val: Option<f64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" | "UNKNOWN" => {
                let val: Option<String> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMP" | "Timestamp" => {
                let val: Option<chrono::NaiveDateTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMPTZ" | "Timestamptz" => {
                let val: Option<chrono::DateTime<chrono::Utc>> =
                    row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "UUID" | "Uuid" => {
                let val: Option<uuid::Uuid> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "JSON" | "JSONB" => {
                let val: Option<serde_json::Value> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INET" | "inet" | "CIDR" | "cidr" => {
                let val: Option<ipnetwork::IpNetwork> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DATE" | "Date" => {
                let val: Option<chrono::NaiveDate> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIME" | "Time" => {
                let val: Option<chrono::NaiveTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            _ => {
                // Try to get as String for anything else
                if let Ok(val) = row.try_get::<String, _>(col_name) {
                    serde_json::Value::String(val)
                } else {
                    serde_json::Value::String(format!("Unsupported Type: {}", type_name))
                }
            }
        };
        values.push(value);
    }
    values
}

fn row_to_json(row: PgRow) -> serde_json::Value {
    let mut row_map = serde_json::Map::new();
    for col in row.columns() {
        let col_name = col.name();
        let type_info = col.type_info();
        let type_name = type_info.name();

        let value = match type_name {
            "BOOL" => {
                let val: Option<bool> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT2" | "INT4" | "INT" => {
                let val: Option<i32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INT8" | "BIGINT" => {
                let val: Option<i64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT4" | "REAL" => {
                let val: Option<f32> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                let val: Option<f64> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "VARCHAR" | "TEXT" | "BPCHAR" | "NAME" | "UNKNOWN" => {
                let val: Option<String> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMP" | "Timestamp" => {
                let val: Option<chrono::NaiveDateTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIMESTAMPTZ" | "Timestamptz" => {
                let val: Option<chrono::DateTime<chrono::Utc>> =
                    row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "UUID" | "Uuid" => {
                let val: Option<uuid::Uuid> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "JSON" | "JSONB" => {
                let val: Option<serde_json::Value> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "INET" | "inet" | "CIDR" | "cidr" => {
                let val: Option<ipnetwork::IpNetwork> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "DATE" | "Date" => {
                let val: Option<chrono::NaiveDate> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            "TIME" | "Time" => {
                let val: Option<chrono::NaiveTime> = row.try_get(col_name).unwrap_or(None);
                serde_json::json!(val)
            }
            _ => {
                if let Ok(val) = row.try_get::<String, _>(col_name) {
                    serde_json::Value::String(val)
                } else {
                    serde_json::Value::String(format!("Unsupported Type: {}", type_name))
                }
            }
        };
        row_map.insert(col_name.to_string(), value);
    }
    serde_json::Value::Object(row_map)
}

#[tauri::command]
pub async fn get_table_data(
    connection_id: String,
    table_name: String,
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    // WARNING: SQL Injection risk. Validate table_name in production.
    // Get total count
    let count_query = format!("SELECT COUNT(*) FROM \"{}\"", table_name);
    let count_row: (i64,) = sqlx::query_as(&count_query).fetch_one(&pool).await?;
    let total_rows = count_row.0;

    // Get data with pagination
    let query = format!("SELECT * FROM \"{}\" LIMIT $1 OFFSET $2", table_name);

    let rows = sqlx::query(&query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&pool)
        .await?;

    let columns = if let Some(first_row) = rows.first() {
        first_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect()
    } else {
        // If no rows, we might want to get columns from structure, but for now empty is fine
        // or we could keep previous columns if we had them.
        // For now, let's just return empty columns if no data.
        Vec::new()
    };

    let result_rows: Vec<Vec<serde_json::Value>> = rows.into_iter().map(row_to_values).collect();

    Ok(QueryResult {
        columns,
        rows: result_rows,
        total_rows: Some(total_rows),
    })
}

#[tauri::command]
pub async fn get_table_structure(
    connection_id: String,
    table_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    let rows = sqlx::query(
        "SELECT column_name, data_type, is_nullable 
         FROM information_schema.columns 
         WHERE table_name = $1 
         ORDER BY ordinal_position;",
    )
    .bind(table_name)
    .fetch_all(&pool)
    .await?;

    let results: Vec<serde_json::Value> = rows.into_iter().map(row_to_json).collect();
    Ok(results)
}

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    // This is for SELECT queries. For others, we might need execute().
    // Assuming the user wants to see results.
    // If it's not a SELECT, fetch_all might return empty or error depending on the query.
    // Ideally we check if it starts with SELECT.

    let rows = sqlx::query(&query).fetch_all(&pool).await?;

    let columns = if let Some(first_row) = rows.first() {
        first_row
            .columns()
            .iter()
            .map(|col| col.name().to_string())
            .collect()
    } else {
        Vec::new()
    };

    let result_rows: Vec<Vec<serde_json::Value>> = rows.into_iter().map(row_to_values).collect();

    Ok(QueryResult {
        columns,
        rows: result_rows,
        total_rows: None,
    })
}

#[tauri::command]
pub async fn get_database_schema(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, DatabaseError> {
    let pool = get_pool(&state, &connection_id)?;

    let rows = sqlx::query(
        "SELECT table_name, column_name FROM information_schema.columns WHERE table_schema = 'public';",
    )
    .fetch_all(&pool)
    .await?;

    let mut schema: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for row in rows {
        let table_name: String = row.get("table_name");
        let column_name: String = row.get("column_name");

        schema
            .entry(table_name)
            .or_insert_with(Vec::new)
            .push(column_name);
    }

    Ok(schema)
}
