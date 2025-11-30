use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::connection_manager::SavedConnection;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStateData {
    pub selected_space_id: Option<String>,
    pub spaces: Vec<SpaceState>,
    pub tabs: Vec<TabState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpaceState {
    pub id: String,
    pub config: SavedConnection,
    pub current_database: String,
    pub active_tab_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TabData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TabState {
    pub id: String,
    pub title: String,
    pub connection_id: String,
    pub database: String,
    #[serde(rename = "type")]
    pub tab_type: String,
    pub data: TabData,
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
