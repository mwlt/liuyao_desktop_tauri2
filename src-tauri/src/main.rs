/*
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-25 18:05:32
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-29 00:03:30
 * @FilePath: \liuyao_desktop_tauri\src-tauri\src\main.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
mod proxy_server;
use proxy_server::{ProxyServer, ProxySettings, ProxyType};
use std::sync::Mutex;
use once_cell::sync::Lazy;
mod read_system_proxy;
use read_system_proxy::get_system_proxy_info;
use serde::Serialize;
use env_logger;

// 全局代理服务器实例
static PROXY_SERVER: Lazy<Mutex<Option<ProxyServer>>> = Lazy::new(|| Mutex::new(None));
static LOCAL_PROXY_PORT: Lazy<Mutex<u16>> = Lazy::new(|| Mutex::new(0));

#[tauri::command]
fn get_local_proxy_port() -> u16 {
    *LOCAL_PROXY_PORT.lock().unwrap()
}

// 新增：更新代理设置命令
#[tauri::command]
fn update_proxy_settings(settings: ProxySettings) -> Result<(), String> {
    println!("[main] 收到代理设置更新请求: {:?}", settings);
    
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            server.update_proxy_settings(settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：获取当前代理设置命令
#[tauri::command]
fn get_proxy_settings() -> Result<ProxySettings, String> {
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            Ok(server.get_proxy_settings())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：设置代理类型命令
#[tauri::command(rename_all = "camelCase")]
fn set_proxy_type(proxy_type: String) -> Result<(), String> {
    let proxy_type = match proxy_type.as_str() {
        "None" => ProxyType::None,
        "System" => ProxyType::System,
        "Manual" => ProxyType::Manual,
        _ => return Err("无效的代理类型".to_string()),
    };

    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.proxy_type = proxy_type.clone();
            
            // 如果是禁用代理，清除所有代理地址并禁用代理
            if proxy_type == ProxyType::None {
                settings.http_proxy = None;
                settings.https_proxy = None;
                settings.socks5_proxy = None;
                settings.enabled = false;
            }
            
            server.update_proxy_settings(settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：设置HTTP代理命令
#[tauri::command]
fn set_http_proxy(proxy: String) -> Result<(), String> {
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.http_proxy = if proxy.is_empty() { None } else { Some(proxy) };
            server.update_proxy_settings(settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：设置HTTPS代理命令
#[tauri::command]
fn set_https_proxy(proxy: String) -> Result<(), String> {
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.https_proxy = if proxy.is_empty() { None } else { Some(proxy) };
            server.update_proxy_settings(settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：设置SOCKS5代理命令
#[tauri::command]
fn set_socks5_proxy(proxy: String) -> Result<(), String> {
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.socks5_proxy = if proxy.is_empty() { None } else { Some(proxy) };
            server.update_proxy_settings(settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：测试代理可用性命令
#[derive(Debug, Serialize)]
struct TestResult {
    proxy_available: bool,
    core333_accessible: bool,
    google_accessible: bool,
    message: String,
}

// 测试代理连接
#[tauri::command]
async fn test_proxy_connectivity(proxy_url: String) -> Result<TestResult, String> {
    println!("[main] 开始测试代理连接: {}", proxy_url);
    
    use std::time::Duration;
    use tokio::time::timeout;
    use reqwest::Client;
    
    // 处理直连模式
    if proxy_url == "direct://" {
        println!("[main] 直连模式测试");
        
        // 创建不使用代理的客户端
        let client = match Client::builder()
            .timeout(Duration::from_secs(10))
            .build() {
            Ok(c) => c,
            Err(e) => return Ok(TestResult {
                proxy_available: true, // 直连模式下始终为 true
                core333_accessible: false,
                google_accessible: false,
                message: format!("创建HTTP客户端失败: {}", e),
            }),
        };
        
        // 测试 core333.com
        let core333_result = match timeout(
            Duration::from_secs(10),
            client.get("http://www.core333.com").send()
        ).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        };
        
        // 测试 google.com
        let google_result = match timeout(
            Duration::from_secs(10),
            client.get("https://www.google.com").send()
        ).await {
            Ok(Ok(response)) => response.status().is_success(),
            _ => false,
        };
        
        return Ok(TestResult {
            proxy_available: true, // 直连模式下始终为 true
            core333_accessible: core333_result,
            google_accessible: google_result,
            message: format!(
                "直连测试结果：core333.com: {}，google.com: {}", 
                if core333_result { "可访问" } else { "不可访问" },
                if google_result { "可访问" } else { "不可访问" }
            ),
        });
    }
    
    // 解析代理地址
    let proxy_parts: Vec<&str> = proxy_url.split("://").collect();
    let (protocol, address) = if proxy_parts.len() == 2 {
        (proxy_parts[0], proxy_parts[1])
    } else {
        ("http", proxy_url.as_str())
    };
    
    let address_parts: Vec<&str> = address.split(':').collect();
    if address_parts.len() != 2 {
        return Ok(TestResult {
            proxy_available: false,
            core333_accessible: false,
            google_accessible: false,
            message: "代理地址格式错误，应为 host:port".to_string(),
        });
    }
    
    let host = address_parts[0];
    let port: u16 = match address_parts[1].parse() {
        Ok(p) => p,
        Err(_) => return Ok(TestResult {
            proxy_available: false,
            core333_accessible: false,
            google_accessible: false,
            message: "端口号格式错误".to_string(),
        }),
    };
    
    println!("[main] 解析代理地址: {}://{}:{}", protocol, host, port);
    
    // 测试TCP连接
    let socket_addr = format!("{}:{}", host, port);
    match timeout(Duration::from_secs(5), tokio::net::TcpStream::connect(&socket_addr)).await {
        Ok(Ok(_)) => {
            println!("[main] ✅ TCP连接成功");
            
            // 创建HTTP客户端
            let client = match Client::builder()
                .proxy(reqwest::Proxy::all(&proxy_url).unwrap())
                .timeout(Duration::from_secs(10))
                .build() {
                Ok(c) => c,
                Err(e) => return Ok(TestResult {
                    proxy_available: true,
                    core333_accessible: false,
                    google_accessible: false,
                    message: format!("创建HTTP客户端失败: {}", e),
                }),
            };
            
            // 测试 core333.com
            let core333_result = match timeout(
                Duration::from_secs(10),
                client.get("http://www.core333.com").send()
            ).await {
                Ok(Ok(response)) => response.status().is_success(),
                _ => false,
            };
            
            // 测试 google.com
            let google_result = match timeout(
                Duration::from_secs(10),
                client.get("https://www.google.com").send()
            ).await {
                Ok(Ok(response)) => response.status().is_success(),
                _ => false,
            };
            
            Ok(TestResult {
                proxy_available: true,
                core333_accessible: core333_result,
                google_accessible: google_result,
                message: format!(
                    "代理可用。core333.com: {}，google.com: {}", 
                    if core333_result { "可访问" } else { "不可访问" },
                    if google_result { "可访问" } else { "不可访问" }
                ),
            })
        },
        _ => Ok(TestResult {
            proxy_available: false,
            core333_accessible: false,
            google_accessible: false,
            message: "无法连接到代理服务器".to_string(),
        }),
    }
}

// 新增：应用系统代理设置命令
#[tauri::command]
fn apply_system_proxy() -> Result<(), String> {
    println!("[main] 开始应用系统代理设置");
    
    let system_proxy = get_system_proxy_info();
    println!("[main] 获取到系统代理信息: {:?}", system_proxy);
    
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.proxy_type = ProxyType::System;
            
            if system_proxy.proxy_enabled {
                // 系统代理已启用，应用系统代理设置
                if !system_proxy.http_proxy.is_empty() {
                    settings.http_proxy = Some(system_proxy.http_proxy.clone());
                    println!("[main] 应用系统HTTP代理: {}", system_proxy.http_proxy);
                }
                
                if !system_proxy.https_proxy.is_empty() {
                    settings.https_proxy = Some(system_proxy.https_proxy.clone());
                    println!("[main] 应用系统HTTPS代理: {}", system_proxy.https_proxy);
                }
                
                if !system_proxy.socks_proxy.is_empty() {
                    settings.socks5_proxy = Some(system_proxy.socks_proxy.clone());
                    println!("[main] 应用系统SOCKS代理: {}", system_proxy.socks_proxy);
                }
                
                settings.enabled = true;
                println!("[main] ✅ 系统代理设置已应用: {:?}", settings);
            } else {
                // 系统代理未启用，但仍然设置为系统代理模式，只是暂时不启用
                settings.http_proxy = None;
                settings.https_proxy = None;
                settings.socks5_proxy = None;
                settings.enabled = false;
                println!("[main] 系统代理未启用，设置为系统代理模式但暂不启用");
            }
            
            server.update_proxy_settings(settings.clone());
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

// 新增：获取当前代理状态命令
#[tauri::command]
fn get_proxy_status() -> Result<String, String> {
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let settings = server.get_proxy_settings();
            let status = format!(
                "代理类型: {:?}, HTTP: {:?}, HTTPS: {:?}, SOCKS5: {:?}, 启用: {}",
                settings.proxy_type,
                settings.http_proxy,
                settings.https_proxy,
                settings.socks5_proxy,
                settings.enabled
            );
            println!("[main] 当前代理状态: {}", status);
            Ok(status)
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

#[tauri::command]
fn apply_manual_proxy() -> Result<(), String> {
    println!("[main] 开始应用手动代理设置");
    
    if let Ok(proxy_server) = PROXY_SERVER.lock() {
        if let Some(server) = proxy_server.as_ref() {
            let mut settings = server.get_proxy_settings();
            settings.proxy_type = ProxyType::Manual;
            settings.enabled = true;  // 启用代理
            
            // 应用设置
            server.update_proxy_settings(settings.clone());
            println!("[main] ✅ 手动代理设置已应用: {:?}", settings);
            Ok(())
        } else {
            Err("代理服务器未启动".to_string())
        }
    } else {
        Err("无法获取代理服务器锁".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 设置日志级别
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    
    // 预设代理环境变量（使用默认端口8080）
    // 如果8080被占用，代理服务器会自动切换到其他端口
    std::env::set_var("HTTP_PROXY", "http://127.0.0.1:8080");
    std::env::set_var("HTTPS_PROXY", "http://127.0.0.1:8080");
    println!("🔧 预设WebView代理环境变量: HTTP_PROXY=http://127.0.0.1:8080");
    
    // 使用 Result 来处理错误
    if let Err(e) = run_app() {
        eprintln!("应用程序运行失败: {}", e);
        std::process::exit(1);
    }
}

fn run_app() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_system_proxy_info,
            get_local_proxy_port,
            // 代理管理命令
            update_proxy_settings,
            get_proxy_settings,
            set_proxy_type,
            set_http_proxy,
            set_https_proxy,
            set_socks5_proxy,
            // 新增的命令
            test_proxy_connectivity,
            apply_system_proxy,
            apply_manual_proxy,
            get_proxy_status
        ])
        .setup(|app| {
            // 在setup中启动本地代理服务器，自动检测端口
            let proxy = ProxyServer::start_auto_port(8080, 8099)
                .ok_or("未能找到可用端口，代理服务器启动失败")?;
            
            println!("本地代理服务器已启动，端口: {}", proxy.port);
            *LOCAL_PROXY_PORT.lock().unwrap() = proxy.port;
            
            // 保存代理服务器实例到全局变量
            *PROXY_SERVER.lock().unwrap() = Some(proxy);
            
            let proxy_port = *LOCAL_PROXY_PORT.lock().unwrap();
            
            // 设置 WebView 代理环境变量
            std::env::set_var("HTTP_PROXY", format!("http://127.0.0.1:{}", proxy_port));
            std::env::set_var("HTTPS_PROXY", format!("http://127.0.0.1:{}", proxy_port));
            println!("✅ WebView代理环境变量已设置: HTTP_PROXY=http://127.0.0.1:{}", proxy_port);
            
            // 获取主窗口并确保它显示
            if let Some(window) = app.get_webview_window("main") {
                // 强制显示窗口
                window.show()?;
                window.unminimize()?;
                window.set_focus()?;
                
                // 居中窗口
                window.center()?;
                
                // 设置窗口置顶（暂时）
                window.set_always_on_top(true)?;
                
                println!("✅ 主窗口已强制显示、置顶、居中并获得焦点");
                
                // 2秒后取消置顶
                let window_clone = window.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if let Err(e) = window_clone.set_always_on_top(false) {
                        eprintln!("取消窗口置顶失败: {}", e);
                    }
                    println!("窗口置顶已取消");
                });
                
                println!("WebView已配置为使用代理: 127.0.0.1:{}", proxy_port);
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .map_err(|e| e.into())
}

fn main() {
    run();
}
