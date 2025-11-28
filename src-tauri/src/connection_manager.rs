use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;
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
    pub ssh_enabled: bool,
    pub ssh_host: Option<String>,
    pub ssh_port: Option<u16>,
    pub ssh_user: Option<String>,
    pub ssh_key_path: Option<String>,
}

pub struct ConnectionManager {
    pub ssh_process: Mutex<Option<Child>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            ssh_process: Mutex::new(None),
        }
    }

    pub fn start_ssh_tunnel(
        &self,
        ssh_host: &str,
        ssh_port: Option<u16>,
        ssh_user: Option<&str>,
        ssh_key_path: Option<&str>,
        remote_host: &str,
        remote_port: u16,
        local_port: u16,
    ) -> Result<(), String> {
        let mut command = Command::new("ssh");

        // Basic SSH flags
        command.arg("-N"); // Do not execute a remote command
        command.arg("-L"); // Local port forwarding
        command.arg(format!("{}:{}:{}", local_port, remote_host, remote_port));

        if let Some(key_path) = ssh_key_path {
            if !key_path.trim().is_empty() {
                command.arg("-i");
                command.arg(key_path);
            }
        }

        if let Some(port) = ssh_port {
            command.arg("-p");
            command.arg(port.to_string());
        }

        let destination = if let Some(user) = ssh_user {
            if !user.trim().is_empty() {
                format!("{}@{}", user, ssh_host)
            } else {
                ssh_host.to_string()
            }
        } else {
            ssh_host.to_string()
        };
        command.arg(destination);

        // Ensure we don't block on host key verification for better UX in this demo
        // WARNING: In a real secure app, we should handle known_hosts properly
        command.arg("-o");
        command.arg("StrictHostKeyChecking=no");
        command.arg("-o");
        command.arg("UserKnownHostsFile=/dev/null");

        let child = command
            .spawn()
            .map_err(|e| format!("Failed to start SSH tunnel: {}", e))?;

        let mut process_guard = self.ssh_process.lock().map_err(|e| e.to_string())?;

        // Kill existing process if any
        if let Some(mut old_child) = process_guard.take() {
            let _ = old_child.kill();
        }

        *process_guard = Some(child);

        // Give it a moment to establish
        std::thread::sleep(std::time::Duration::from_millis(1000));

        Ok(())
    }

    pub fn stop_ssh_tunnel(&self) -> Result<(), String> {
        let mut process_guard = self.ssh_process.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = process_guard.take() {
            child
                .kill()
                .map_err(|e| format!("Failed to kill SSH process: {}", e))?;
        }
        Ok(())
    }
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
