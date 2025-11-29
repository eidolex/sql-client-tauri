use crate::connection_manager::SavedConnection;
use crate::database_provider::DatabaseProvider;
use crate::mysql_provider::MysqlProvider;
use crate::postgres_provider::PostgresProvider;
use crate::ssh_tunnel::{SshTunnel, TunnelConfig};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};
use tauri::State;

pub struct DbConnection {
    pub provider: Arc<dyn DatabaseProvider + Send + Sync>,
    pub _ssh_tunnel: Option<Arc<SshTunnel>>,
}

pub struct AppState {
    pub connections: Mutex<HashMap<String, DbConnection>>,
    pub tunnels: Mutex<HashMap<TunnelConfig, Weak<SshTunnel>>>,
}

#[derive(serde::Serialize, Debug)]
pub struct DatabaseError {
    pub message: String,
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

    // 2. Construct Connection String and Connect
    let password = connection_config.password.as_deref().unwrap_or("");

    let provider: Arc<dyn DatabaseProvider + Send + Sync> = match connection_config.db_type.as_str()
    {
        "mysql" => {
            let connection_string = format!(
                "mysql://{}:{}@{}:{}/{}",
                connection_config.username, password, db_host, db_port, connection_config.database
            );
            Arc::new(MysqlProvider::new(&connection_string).await?)
        }
        _ => {
            // Default to Postgres
            let connection_string = format!(
                "postgres://{}:{}@{}:{}/{}",
                connection_config.username, password, db_host, db_port, connection_config.database
            );
            Arc::new(PostgresProvider::new(&connection_string).await?)
        }
    };

    let connection_id = uuid::Uuid::new_v4().to_string();
    let connection = DbConnection {
        provider,
        _ssh_tunnel: ssh_tunnel,
    };

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
        conn.provider.close().await;

        // Clean up dead tunnel references
        // When the connection is dropped, the Arc<SshTunnel> is dropped.
        // If this was the last reference, the tunnel will be stopped automatically.
        // We clean up the Weak references that can no longer be upgraded.
        let mut tunnels = state.tunnels.lock().unwrap();
        tunnels.retain(|_, weak_tunnel| weak_tunnel.upgrade().is_some());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_databases(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.list_databases().await
}

#[tauri::command]
pub async fn list_tables(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.list_tables().await
}

#[derive(serde::Serialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: Option<i64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Filter {
    pub field: String,
    pub operator: String,
    pub value: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sort {
    pub field: String,
    pub order: String,
}

#[tauri::command]
pub async fn get_table_data(
    connection_id: String,
    table_name: String,
    limit: i64,
    offset: i64,
    filters: Vec<Filter>,
    sorts: Vec<Sort>,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider
        .get_table_data(table_name, limit, offset, filters, sorts)
        .await
}

#[derive(serde::Serialize)]
pub struct ColumnDefinition {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: String,
    pub column_default: Option<String>,
    pub comment: Option<String>,
    pub foreign_key: Option<String>,
}

#[tauri::command]
pub async fn get_table_structure(
    connection_id: String,
    table_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<ColumnDefinition>, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.get_table_structure(table_name).await
}

#[derive(serde::Serialize)]
pub struct IndexDefinition {
    pub index_name: String,
    pub index_algorithm: String,
    pub is_unique: bool,
    pub is_primary: bool,
    pub column_names: String,
    pub condition: Option<String>,
    pub comment: Option<String>,
}

#[tauri::command]
pub async fn get_table_indexes(
    connection_id: String,
    table_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<IndexDefinition>, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.get_table_indexes(table_name).await
}

#[tauri::command]
pub async fn execute_query(
    connection_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<QueryResult, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.execute_query(query).await
}

#[tauri::command]
pub async fn get_database_schema(
    connection_id: String,
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, DatabaseError> {
    let provider = {
        let connections = state.connections.lock().unwrap();
        let conn = connections.get(&connection_id).ok_or(DatabaseError {
            message: "Connection not found".to_string(),
        })?;
        conn.provider.clone()
    };
    provider.get_database_schema().await
}
