// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use tauri::Manager;
mod proxy_server;
use proxy_server::ProxyServer;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static LOCAL_PROXY_PORT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));

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
    // 这里我们返回一个空的代理信息，你需要根据实际情况实现系统代理信息的获取
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
fn get_local_proxy_port() -> u16 {
    *LOCAL_PROXY_PORT.lock().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_system_proxy_info, get_local_proxy_port])
        .setup(|app| {
            // 在setup中启动本地代理服务器，自动检测端口
            let proxy = ProxyServer::start_auto_port(8080, 8099);
            let proxy_port = match proxy {
                Some(server) => {
                    println!("本地代理服务器已启动，端口: {}", server.port);
                    *LOCAL_PROXY_PORT.lock().unwrap() = server.port;
                    server.port
                },
                None => {
                    println!("未能找到可用端口，代理服务器启动失败");
                    return Err("代理服务器启动失败".into());
                }
            };
            
            // 设置 WebView 代理环境变量
            std::env::set_var("HTTP_PROXY", format!("http://127.0.0.1:{}", proxy_port));
            std::env::set_var("HTTPS_PROXY", format!("http://127.0.0.1:{}", proxy_port));
            println!("✅ WebView代理环境变量已设置: HTTP_PROXY=http://127.0.0.1:{}", proxy_port);
            
            // 获取主窗口并确保它显示
            if let Some(window) = app.get_webview_window("main") {
                // 强制显示窗口
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                
                // 居中窗口
                let _ = window.center();
                
                // 设置窗口置顶（暂时）
                let _ = window.set_always_on_top(true);
                
                println!("✅ 主窗口已强制显示、置顶、居中并获得焦点");
                
                // 2秒后取消置顶
                let window_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let _ = window_clone.set_always_on_top(false);
                    println!("窗口置顶已取消");
                });
                
                // 设置WebView代理（如果支持的话）
                println!("WebView已配置为使用代理: 127.0.0.1:{}", proxy_port);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    // 预设代理环境变量（使用默认端口8080）
    // 如果8080被占用，代理服务器会自动切换到其他端口
    std::env::set_var("HTTP_PROXY", "http://127.0.0.1:8080");
    std::env::set_var("HTTPS_PROXY", "http://127.0.0.1:8080");
    println!("🔧 预设WebView代理环境变量: HTTP_PROXY=http://127.0.0.1:8080");
    
    run();
}
