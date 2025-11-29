use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedConnection {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub database: String,
    #[serde(default = "default_db_type")]
    pub db_type: String,
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_password: Option<String>,
    pub ssh_key_path: Option<String>,
}

fn default_db_type() -> String {
    "postgres".to_string()
}

fn get_connections_file_path(app_handle: &AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .expect("failed to get app data dir");
    fs::create_dir_all(&path).expect("failed to create app data dir");
    path.push("connections.json");
    path
}

#[tauri::command]
pub fn save_connection(app_handle: AppHandle, connection: SavedConnection) -> Result<(), String> {
    let path = get_connections_file_path(&app_handle);
    let mut connections = load_connections(app_handle.clone()).unwrap_or_default();

    // Update existing or add new
    if let Some(idx) = connections.iter().position(|c| c.id == connection.id) {
        connections[idx] = connection;
    } else {
        connections.push(connection);
    }

    let json = serde_json::to_string_pretty(&connections).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn load_connections(app_handle: AppHandle) -> Result<Vec<SavedConnection>, String> {
    let path = get_connections_file_path(&app_handle);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let connections: Vec<SavedConnection> =
        serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(connections)
}

#[tauri::command]
pub fn delete_connection(app_handle: AppHandle, id: String) -> Result<(), String> {
    let path = get_connections_file_path(&app_handle);
    let mut connections = load_connections(app_handle.clone()).unwrap_or_default();

    connections.retain(|c| c.id != id);

    let json = serde_json::to_string_pretty(&connections).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())?;
    Ok(())
}
