// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod connection_manager;
mod db;

use connection_manager::ConnectionManager;
use db::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            pool: std::sync::Mutex::new(None),
            connection_manager: ConnectionManager::new(),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            db::connect_db,
            db::list_databases,
            db::list_tables,
            db::get_table_data,
            db::get_table_structure,
            db::execute_query,
            db::get_database_schema,
            connection_manager::save_connection,
            connection_manager::load_connections,
            connection_manager::delete_connection
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
