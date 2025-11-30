// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

pub mod connection_manager;
pub mod database_provider;
pub mod db;
pub mod mysql_provider;
pub mod postgres_provider;
pub mod ssh_tunnel;
pub mod state;

use db::AppState;
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState {
            connections: Mutex::new(HashMap::new()),
            tunnels: Mutex::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            db::connect_db,
            db::disconnect_db,
            db::list_databases,
            db::list_tables,
            db::get_table_data,
            db::get_table_structure,
            db::get_table_indexes,
            db::execute_query,
            db::get_database_schema,
            connection_manager::save_connection,
            connection_manager::load_connections,
            connection_manager::delete_connection,
            state::save_app_state,
            state::load_app_state
        ])
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Prevent the window from closing
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
