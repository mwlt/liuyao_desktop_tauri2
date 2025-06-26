use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SystemProxyInfo {
    http_proxy: String,
    https_proxy: String,
    socks_proxy: String,
    ftp_proxy: String,
    no_proxy: String,
    proxy_enabled: bool,
}

#[tauri::command]
fn get_system_proxy_info() -> SystemProxyInfo {
    SystemProxyInfo {
        http_proxy: std::env::var("HTTP_PROXY").unwrap_or_default(),
        https_proxy: std::env::var("HTTPS_PROXY").unwrap_or_default(),
        socks_proxy: std::env::var("SOCKS_PROXY").unwrap_or_default(),
        ftp_proxy: std::env::var("FTP_PROXY").unwrap_or_default(),
        no_proxy: std::env::var("NO_PROXY").unwrap_or_default(),
        proxy_enabled: false,
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet, get_system_proxy_info])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
