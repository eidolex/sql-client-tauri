use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStateData {
    pub spaces: Vec<SpaceState>,
    pub selected_connection_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceState {
    pub id: String,
    pub config_id: String, // Reference to saved connection
    pub current_database: String,
    pub active_tab_id: Option<String>,
    pub tabs: Vec<TabState>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TabState {
    #[serde(rename = "data")]
    Data {
        id: String,
        title: String,
        connection_id: String,
        database: String,
        table: String,
        page: i32,
        page_size: i32,
    },
    #[serde(rename = "structure")]
    Structure {
        id: String,
        title: String,
        connection_id: String,
        database: String,
        table: String,
    },
    #[serde(rename = "query")]
    Query {
        id: String,
        title: String,
        connection_id: String,
        database: String,
        query: Option<String>,
    },
}

fn get_state_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;

    // Ensure the directory exists
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    Ok(app_data_dir.join("app_state.json"))
}

#[tauri::command]
pub async fn save_app_state(app: AppHandle, state: AppStateData) -> Result<(), String> {
    let state_file = get_state_file_path(&app)?;

    let json = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize state: {}", e))?;

    fs::write(&state_file, json).map_err(|e| format!("Failed to write state file: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn load_app_state(app: AppHandle) -> Result<Option<AppStateData>, String> {
    let state_file = get_state_file_path(&app)?;

    // If the file doesn't exist, return None
    if !state_file.exists() {
        return Ok(None);
    }

    let json =
        fs::read_to_string(&state_file).map_err(|e| format!("Failed to read state file: {}", e))?;

    let state: AppStateData =
        serde_json::from_str(&json).map_err(|e| format!("Failed to deserialize state: {}", e))?;

    Ok(Some(state))
}
