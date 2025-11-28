use crate::connection_manager::{ConnectionManager, SavedConnection};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Column, Pool, Postgres, Row, TypeInfo};
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub pool: Mutex<Option<Pool<Postgres>>>,
    pub connection_manager: ConnectionManager,
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

#[tauri::command]
pub async fn connect_db(
    connection_config: SavedConnection,
    state: State<'_, AppState>,
) -> Result<String, DatabaseError> {
    // 1. Handle SSH Tunnel if enabled
    let mut db_host = connection_config.host.clone();
    let mut db_port = connection_config.port;

    if connection_config.ssh_enabled {
        if let Some(ssh_host) = &connection_config.ssh_host {
            let ssh_port = connection_config.ssh_port;
            let ssh_user = connection_config.ssh_user.as_deref();
            let local_port = 5433; // Fixed local port for now, could be dynamic

            state
                .connection_manager
                .start_ssh_tunnel(
                    ssh_host,
                    ssh_port,
                    ssh_user,
                    connection_config.ssh_key_path.as_deref(),
                    &db_host,
                    db_port,
                    local_port,
                )
                .map_err(|e| DatabaseError { message: e })?;

            // Update connection info to point to local tunnel
            db_host = "127.0.0.1".to_string();
            db_port = local_port;
        }
    } else {
        // Ensure any previous tunnel is stopped if we are connecting directly now
        let _ = state.connection_manager.stop_ssh_tunnel();
    }

    // 2. Construct Connection String
    let password = connection_config.password.as_deref().unwrap_or("");
    let connection_string = format!(
        "postgres://{}:{}@{}:{}/{}",
        connection_config.username, password, db_host, db_port, connection_config.database
    );

    // 3. Connect to Database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&connection_string)
        .await?;

    *state.pool.lock().unwrap() = Some(pool);

    Ok("Connected successfully".to_string())
}

#[tauri::command]
pub async fn list_databases(state: State<'_, AppState>) -> Result<Vec<String>, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

    let rows = sqlx::query("SELECT datname FROM pg_database WHERE datistemplate = false;")
        .fetch_all(&pool)
        .await?;

    let databases: Vec<String> = rows.iter().map(|row| row.get("datname")).collect();
    Ok(databases)
}

#[tauri::command]
pub async fn list_tables(state: State<'_, AppState>) -> Result<Vec<String>, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

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
    table_name: String,
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

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
    table_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

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
    query: String,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

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
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, DatabaseError> {
    let pool = {
        let pool_guard = state.pool.lock().unwrap();
        pool_guard.as_ref().cloned().ok_or(DatabaseError {
            message: "Not connected to database".to_string(),
        })?
    };

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
