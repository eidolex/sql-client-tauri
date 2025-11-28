// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod db;
use db::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            pool: Mutex::new(None),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            db::connect_db,
            db::list_databases,
            db::list_tables,
            db::get_table_data,
            db::get_table_structure,
            db::execute_query
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
