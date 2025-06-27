/*
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-26 00:11:01
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-27 16:52:44
 * @FilePath: \liuyao_desktop_tauri\src-tauri\src\proxy_server.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::io::{Read, Write};
use std::time::Duration;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer
const TIMEOUT: u64 = 60; // 60秒超时

// 新增：代理配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub proxy_type: ProxyType,
    pub http_proxy: Option<String>,
    pub https_proxy: Option<String>,
    pub socks5_proxy: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    None,       // 不使用代理
    System,     // 使用系统代理
    Http,       // HTTP代理
    Https,      // HTTPS代理
    Socks5,     // SOCKS5代理
    Manual,     // 手动配置（多种代理类型）
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::None,
            http_proxy: None,
            https_proxy: None,
            socks5_proxy: None,
            username: None,
            password: None,
            enabled: true,
        }
    }
}

// 获取系统代理设置
#[derive(Debug)]
struct SystemProxyConfig {
    enabled: bool,
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    bypass_list: Vec<String>,
    pac_url: Option<String>,
}

impl Default for SystemProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            http_proxy: None,
            https_proxy: None,
            bypass_list: Vec::new(),
            pac_url: None,
        }
    }
}

fn get_system_proxy_config() -> SystemProxyConfig {
    let mut config = SystemProxyConfig::default();

    // 检查环境变量
    if let Ok(proxy) = std::env::var("http_proxy").or_else(|_| std::env::var("HTTP_PROXY")) {
        config.http_proxy = Some(proxy);
        config.enabled = true;
    }
    if let Ok(proxy) = std::env::var("https_proxy").or_else(|_| std::env::var("HTTPS_PROXY")) {
        config.https_proxy = Some(proxy);
        config.enabled = true;
    }
    if let Ok(no_proxy) = std::env::var("no_proxy").or_else(|_| std::env::var("NO_PROXY")) {
        config.bypass_list = no_proxy.split(',').map(|s| s.to_string()).collect();
    }

    // Windows 系统代理设置
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        if let Ok(internet_settings) = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        {
            // 检查代理是否启用
            if let Ok(proxy_enable) = internet_settings.get_value::<u32, _>("ProxyEnable") {
                config.enabled = proxy_enable == 1;

                // 读取代理服务器地址
                if let Ok(proxy_server) = internet_settings.get_value::<String, _>("ProxyServer") {
                    if proxy_server.contains("=") {
                        // 分别设置的代理格式: "http=proxy:port;https=proxy:port;..."
                        for proxy in proxy_server.split(';') {
                            let parts: Vec<&str> = proxy.split('=').collect();
                            if parts.len() == 2 {
                                match parts[0] {
                                    "http" => config.http_proxy = Some(format!("http://{}", parts[1])),
                                    "https" => config.https_proxy = Some(format!("http://{}", parts[1])),
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        // 所有协议使用相同的代理
                        let proxy = format!("http://{}", proxy_server);
                        config.http_proxy = Some(proxy.clone());
                        config.https_proxy = Some(proxy);
                    }
                }

                // 读取绕过列表
                if let Ok(proxy_override) = internet_settings.get_value::<String, _>("ProxyOverride") {
                    config.bypass_list = proxy_override
                        .split(';')
                        .map(|s| s.to_string())
                        .collect();
                }

                // 读取自动配置脚本URL
                if let Ok(auto_config_url) = internet_settings.get_value::<String, _>("AutoConfigURL") {
                    config.pac_url = Some(auto_config_url);
                }
            }
        }
    }

    config
}

// 检查是否应该绕过代理
fn should_bypass_proxy(host: &str, config: &SystemProxyConfig) -> bool {
    if !config.enabled {
        return true;
    }

    for bypass in &config.bypass_list {
        if bypass == "<local>" && (host.contains("localhost") || host.contains("127.0.0.1")) {
            return true;
        }
        if let Some(pattern) = bypass.strip_prefix('*') {
            if host.ends_with(pattern) {
                return true;
            }
        } else if host == bypass {
            return true;
        }
    }
    false
}

// 获取系统HTTP代理设置
fn get_system_http_proxy() -> Option<String> {
    let config = get_system_proxy_config();
    if config.enabled {
        config.http_proxy
    } else {
        None
    }
}

// 获取系统HTTPS代理设置
fn get_system_https_proxy() -> Option<String> {
    let config = get_system_proxy_config();
    if config.enabled {
        config.https_proxy.or(config.http_proxy)
    } else {
        None
    }
}

pub struct ProxyServer {
    pub port: u16,
    pub settings: Arc<Mutex<ProxySettings>>, // 新增：代理配置
}

// 强制直连函数，绕过系统代理
fn direct_connect(addr: &str) -> std::io::Result<TcpStream> {
    println!("[proxy] 尝试直连到: {}", addr);
    
    let socket_addrs: Vec<_> = addr.to_socket_addrs()?.collect();
    if socket_addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "无法解析地址"
        ));
    }
    
    for socket_addr in socket_addrs {
        match TcpStream::connect_timeout(&socket_addr, Duration::from_secs(TIMEOUT)) {
            Ok(stream) => {
                println!("[proxy] ✅ 直连成功: {} -> {}", addr, socket_addr);
                return Ok(stream);
            }
            Err(e) => {
                println!("[proxy] ❌ 直连失败: {} -> {} ({})", addr, socket_addr, e);
                continue;
            }
        }
    }
    
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionRefused,
        "所有地址连接失败"
    ))
}

// 新增：根据配置选择连接方式
fn connect_with_proxy_settings(target: &str, settings: &ProxySettings) -> std::io::Result<TcpStream> {
    // 防止循环代理：如果目标是本地代理端口，直接连接
    if target.contains("127.0.0.1:8080") || target.contains("localhost:8080") {
        println!("[proxy] 检测到循环代理，改为直连: {}", target);
        return direct_connect(target);
    }

    match &settings.proxy_type {
        ProxyType::None => direct_connect(target),
        ProxyType::System => {
            let config = get_system_proxy_config();
            
            // 解析目标主机名
            let host = target.split(':').next().unwrap_or(target);
            
            // 检查是否应该绕过代理
            if should_bypass_proxy(host, &config) {
                println!("[proxy] 目标在代理绕过列表中，直连: {}", target);
                return direct_connect(target);
            }

            // 根据目标协议选择代理
            let proxy = if target.starts_with("https://") {
                get_system_https_proxy()
            } else {
                get_system_http_proxy()
            };

            if let Some(proxy) = proxy {
                // 检查系统代理是否指向自己
                if proxy.contains("127.0.0.1:8080") || proxy.contains("localhost:8080") {
                    println!("[proxy] 系统代理指向自己，改为直连: {}", target);
                    direct_connect(target)
                } else {
                    proxy_connect(target, &proxy)
                }
            } else {
                println!("[proxy] 系统代理未设置，返回错误");
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "系统代理未设置"
                ))
            }
        },
        ProxyType::Http => {
            if let Some(proxy) = &settings.http_proxy {
                // 检查HTTP代理是否指向自己
                if proxy.contains("127.0.0.1:8080") || proxy.contains("localhost:8080") {
                    println!("[proxy] HTTP代理指向自己，改为直连: {}", target);
                    direct_connect(target)
                } else {
                    proxy_connect(target, proxy)
                }
            } else {
                direct_connect(target)
            }
        },
        ProxyType::Https => {
            if let Some(proxy) = &settings.https_proxy {
                // 检查HTTPS代理是否指向自己
                if proxy.contains("127.0.0.1:8080") || proxy.contains("localhost:8080") {
                    println!("[proxy] HTTPS代理指向自己，改为直连: {}", target);
                    direct_connect(target)
                } else {
                    proxy_connect(target, proxy)
                }
            } else {
                direct_connect(target)
            }
        },
        ProxyType::Socks5 => {
            if let Some(proxy) = &settings.socks5_proxy {
                socks5_connect(target, proxy, &settings.username, &settings.password)
            } else {
                direct_connect(target)
            }
        },
        ProxyType::Manual => {
            // 手动模式：根据目标协议选择合适的代理
            if target.starts_with("https://") {
                if let Some(proxy) = &settings.https_proxy {
                    if proxy.contains("127.0.0.1:8080") || proxy.contains("localhost:8080") {
                        return direct_connect(target);
                    }
                    return proxy_connect(target, proxy);
                }
            }
            if let Some(proxy) = &settings.http_proxy {
                if proxy.contains("127.0.0.1:8080") || proxy.contains("localhost:8080") {
                    direct_connect(target)
                } else {
                    proxy_connect(target, proxy)
                }
            } else {
                direct_connect(target)
            }
        }
    }
}

// 修复：正确的HTTP代理连接实现
fn proxy_connect(target: &str, proxy: &str) -> std::io::Result<TcpStream> {
    println!("[proxy] 通过HTTP代理连接: {} -> {}", target, proxy);
    
    // 解析代理地址
    let proxy_url = if proxy.starts_with("http://") {
        &proxy[7..]
    } else {
        proxy
    };
    
    // 连接到代理服务器
    let mut proxy_stream = TcpStream::connect(proxy_url)?;
    
    // 发送CONNECT请求到代理服务器
    let connect_request = format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n", target, target);
    proxy_stream.write_all(connect_request.as_bytes())?;
    
    // 读取代理响应
    let mut response = [0u8; 1024];
    let bytes_read = proxy_stream.read(&mut response)?;
    let response_str = String::from_utf8_lossy(&response[..bytes_read]);
    
    if response_str.contains("200 Connection established") {
        println!("[proxy] HTTP代理隧道建立成功: {}", target);
        Ok(proxy_stream)
    } else {
        println!("[proxy] HTTP代理响应: {}", response_str.trim());
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "HTTP proxy connection failed"
        ))
    }
}

// 新增：基础SOCKS5连接实现
fn socks5_connect(target: &str, proxy: &str, username: &Option<String>, password: &Option<String>) -> std::io::Result<TcpStream> {
    println!("[proxy] 通过SOCKS5代理连接: {} -> {}", target, proxy);
    
    let mut stream = TcpStream::connect(proxy)?;
    
    // SOCKS5握手
    socks5_handshake(&mut stream, username, password)?;
    
    // SOCKS5连接目标
    socks5_connect_target(&mut stream, target)?;
    
    Ok(stream)
}

// SOCKS5协议握手
fn socks5_handshake(stream: &mut TcpStream, username: &Option<String>, password: &Option<String>) -> std::io::Result<()> {
    // 基础认证方法协商
    let mut auth_methods = vec![0x00]; // 不需要认证
    if username.is_some() && password.is_some() {
        auth_methods.push(0x02); // 用户名密码认证
    }
    
    // 发送握手请求
    let handshake = [
        0x05, // SOCKS版本
        auth_methods.len() as u8,
        auth_methods[0],
    ];
    stream.write_all(&handshake)?;
    
    // 读取响应
    let mut response = [0u8; 2];
    stream.read_exact(&mut response)?;
    
    // 处理认证
    match response[1] {
        0x00 => Ok(()), // 无需认证
        0x02 => socks5_auth(stream, username, password), // 用户名密码认证
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported authentication method"
        )),
    }
}

// SOCKS5用户名密码认证
fn socks5_auth(stream: &mut TcpStream, username: &Option<String>, password: &Option<String>) -> std::io::Result<()> {
    if let (Some(user), Some(pass)) = (username, password) {
        // 发送认证信息
        let mut auth_req = vec![0x01]; // 认证子版本
        auth_req.push(user.len() as u8);
        auth_req.extend(user.as_bytes());
        auth_req.push(pass.len() as u8);
        auth_req.extend(pass.as_bytes());
        
        stream.write_all(&auth_req)?;
        
        // 读取认证响应
        let mut response = [0u8; 2];
        stream.read_exact(&mut response)?;
        
        if response[1] == 0x00 {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "SOCKS5 authentication failed"
            ))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Missing credentials for SOCKS5 authentication"
        ))
    }
}

// SOCKS5连接目标
fn socks5_connect_target(stream: &mut TcpStream, target: &str) -> std::io::Result<()> {
    // 解析目标地址和端口
    let (host, port) = parse_target(target)?;
    
    // 构建连接请求
    let mut request = vec![
        0x05, // SOCKS版本
        0x01, // 连接命令
        0x00, // 保留字节
        0x03, // 域名类型
    ];
    
    request.push(host.len() as u8);
    request.extend(host.as_bytes());
    request.extend(&(port as u16).to_be_bytes());
    
    stream.write_all(&request)?;
    
    // 读取响应
    let mut response = [0u8; 4];
    stream.read_exact(&mut response)?;
    
    if response[1] == 0x00 {
        // 跳过剩余的地址数据
        let mut addr_type = [0u8; 1];
        stream.read_exact(&mut addr_type)?;
        
        match addr_type[0] {
            0x01 => { // IPv4
                let mut addr = [0u8; 6];
                stream.read_exact(&mut addr)?;
            }
            0x03 => { // 域名
                let mut len = [0u8; 1];
                stream.read_exact(&mut len)?;
                let mut addr = vec![0u8; len[0] as usize + 2];
                stream.read_exact(&mut addr)?;
            }
            0x04 => { // IPv6
                let mut addr = [0u8; 18];
                stream.read_exact(&mut addr)?;
            }
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid address type in response"
            )),
        }
        
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "SOCKS5 connection failed"
        ))
    }
}

// 辅助函数：解析目标地址
fn parse_target(target: &str) -> std::io::Result<(String, u16)> {
    if let Some(idx) = target.rfind(':') {
        let host = target[..idx].to_string();
        if let Ok(port) = target[idx+1..].parse::<u16>() {
            Ok((host, port))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid port number"
            ))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Missing port in target address"
        ))
    }
}

fn tunnel(mut a: TcpStream, mut b: TcpStream) {
    let timeout = Duration::from_secs(TIMEOUT);
    let _ = a.set_read_timeout(Some(timeout));
    let _ = a.set_write_timeout(Some(timeout));
    let _ = b.set_read_timeout(Some(timeout));
    let _ = b.set_write_timeout(Some(timeout));

    let mut a2b = a.try_clone().unwrap();
    let mut b2a = b.try_clone().unwrap();
    
    let t1 = thread::spawn(move || {
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            match a2b.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if b.write_all(&buf[..n]).is_err() || b.flush().is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    let t2 = thread::spawn(move || {
        let mut buf = vec![0u8; BUFFER_SIZE];
        loop {
            match b2a.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if a.write_all(&buf[..n]).is_err() || a.flush().is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
    
    let _ = t1.join();
    let _ = t2.join();
}

fn handle_client(mut client_stream: TcpStream, settings: Arc<Mutex<ProxySettings>>) {
    let timeout = Duration::from_secs(TIMEOUT);
    let _ = client_stream.set_read_timeout(Some(timeout));
    let _ = client_stream.set_write_timeout(Some(timeout));

    let mut buffer = vec![0; BUFFER_SIZE];
    let Ok(size) = client_stream.read(&mut buffer) else { return };
    if size == 0 { return; }
    
    let request = String::from_utf8_lossy(&buffer[..size]);
    let mut lines = request.lines();
    let Some(request_line) = lines.next() else { return };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 { return; }
    
    println!("[proxy] 请求: {}", request_line);
    
    // 检查是否是WebSocket升级请求
    let is_websocket = request.to_lowercase().contains("upgrade: websocket");
    if is_websocket {
        println!("[proxy] 检测到WebSocket连接请求");
    }
    
    if parts[0].eq_ignore_ascii_case("CONNECT") {
        let host_port = parts[1];
        let mut host = host_port;
        let mut port = 443u16;
        if let Some(idx) = host_port.find(':') {
            host = &host_port[..idx];
            if let Ok(p) = host_port[idx+1..].parse() {
                port = p;
            }
        }

        let target_addr = format!("{}:{}", host, port);
        println!("[proxy] CONNECT请求到: {}", target_addr);
        
        // 检查是否可能是WebSocket连接（通常通过8000端口）
        if port == 8000 {
            println!("[proxy] 可能是WebSocket CONNECT请求");
        }
        
        let proxy_settings = settings.lock().unwrap().clone();
        
        match connect_with_proxy_settings(&target_addr, &proxy_settings) {
                Ok(target_stream) => {
                println!("[proxy] CONNECT隧道建立成功: {}", target_addr);
                    let _ = client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n");
                    tunnel(client_stream, target_stream);
                }
            Err(e) => {
                println!("[proxy] CONNECT隧道建立失败: {} - {}", target_addr, e);
                    let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
            }
        }
        return;
    }
    
    // 处理HTTP请求
    let mut url = parts[1].to_string();
    
    // 保留现有的URL重写规则
    if url.contains("/api/admin/") {
        url = url.replace("/api/admin/", "/admin/");
        println!("[proxy] URL重写: {} -> {}", parts[1], url);
    }
    
    // 如果是绝对URL
    if url.starts_with("http://") || url.starts_with("ws://") || url.starts_with("wss://") {
        let is_ws = url.starts_with("ws://");
        let is_wss = url.starts_with("wss://");
        let is_websocket = is_ws || is_wss;
        
        let url = &url[if is_wss { 6 } else if is_ws { 5 } else { 7 }..]; // wss:// 是6个字符, ws:// 是5个字符, http:// 是7个字符
        let host_end = url.find('/').unwrap_or(url.len());
        let host_port = &url[..host_end];
        let mut host = host_port;
        let mut port = if is_wss { 443u16 } else if is_ws { 8000u16 } else { 80u16 }; // wss默认443, ws默认8000, http默认80
        if let Some(idx) = host_port.find(':') {
            host = &host_port[..idx];
            if let Ok(p) = host_port[idx+1..].parse() {
                port = p;
            }
        }
        
        if is_websocket {
            println!("[proxy] WebSocket连接到: {}:{} ({})", host, port, if is_wss { "WSS" } else { "WS" });
        }
        
        let target_addr = format!("{}:{}", host, port);
        let proxy_settings = settings.lock().unwrap().clone();
        
        let mut target_stream = match connect_with_proxy_settings(&target_addr, &proxy_settings) {
                Ok(stream) => stream,
            Err(e) => {
                println!("[proxy] 连接失败: {}", e);
                            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                            return;
                        }
        };

        // 对于WebSocket连接，使用更长的超时时间
        let ws_timeout = if is_websocket { Duration::from_secs(300) } else { timeout }; // WebSocket 5分钟超时
        let _ = target_stream.set_read_timeout(Some(ws_timeout));
        let _ = target_stream.set_write_timeout(Some(ws_timeout));

        // 构建新的请求（保留现有的URL重写逻辑）
        let mut modified_request = String::new();
        let mut is_first_line = true;

        for line in request.lines() {
            if is_first_line {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mut path = parts[1].to_string();
                    if path.contains("/api/admin/") {
                        path = path.replace("/api/admin/", "/admin/");
                    }
                    // 对于WebSocket，需要保留完整的路径
                    if is_websocket {
                        // 提取路径部分（去掉协议和主机）
                        if let Some(slash_pos) = url.find('/') {
                            path = url[slash_pos..].to_string();
                        } else {
                            path = "/".to_string();
                        }
                    }
                    modified_request.push_str(&format!("{} {} {}\r\n", parts[0], path, parts.get(2).unwrap_or(&"HTTP/1.1")));
                }
                is_first_line = false;
            } else {
                // 对于WebSocket，保留所有重要的头部
                if is_websocket || (!line.to_lowercase().starts_with("accept-encoding:") &&
                   !line.to_lowercase().starts_with("proxy-connection:")) {
                    modified_request.push_str(line);
                    modified_request.push_str("\r\n");
                }
            }
        }
        modified_request.push_str("\r\n");

        // 发送请求
        if target_stream.write_all(modified_request.as_bytes()).is_err() {
            println!("[proxy] 发送请求失败");
            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
            return;
        }

        // 转发响应
        if is_websocket {
            println!("[proxy] 开始WebSocket数据转发");
        }
        tunnel(client_stream, target_stream);
    } 
    // 相对路径请求
    else if url.starts_with("/") {
        let mut target_host = "localhost:1420";
        for line in request.lines() {
            if line.to_lowercase().starts_with("host:") {
                if let Some(colon_pos) = line.find(':') {
                    let host_value = line[colon_pos + 1..].trim();
                    if !host_value.is_empty() && !host_value.contains(":8080") {
                        target_host = host_value;
                    }
                    break;
                }
            }
        }
        
        println!("[proxy] 相对路径请求 {} 转发到: {}", url, target_host);
        if is_websocket {
            println!("[proxy] 相对路径WebSocket请求");
        }
        
        if let Ok(mut server_stream) = TcpStream::connect(target_host) {
            // 对于WebSocket连接，使用更长的超时时间
            let ws_timeout = if is_websocket { Duration::from_secs(300) } else { timeout };
            let _ = server_stream.set_read_timeout(Some(ws_timeout));
            let _ = server_stream.set_write_timeout(Some(ws_timeout));
            let _ = server_stream.write_all(request.as_bytes());
            
            if is_websocket {
                println!("[proxy] 开始相对路径WebSocket数据转发");
            }
            tunnel(client_stream, server_stream);
        } else {
            println!("[proxy] 连接失败: {}", target_host);
            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
        }
    }
}

impl ProxyServer {
    pub fn start_auto_port(range_start: u16, range_end: u16) -> Option<Self> {
        let settings = Arc::new(Mutex::new(ProxySettings::default()));
        
        for port in range_start..=range_end {
            let addr = format!("127.0.0.1:{}", port);
            if let Ok(listener) = TcpListener::bind(&addr) {
                println!("[proxy] 监听端口: {}", port);
                let settings_clone = Arc::clone(&settings);
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        if let Ok(client_stream) = stream {
                            let settings_clone = Arc::clone(&settings_clone);
                            thread::spawn(move || handle_client(client_stream, settings_clone));
                        }
                    }
                });
                return Some(ProxyServer { port, settings });
            }
        }
        None
    }

    // 新增：更新代理设置
    pub fn update_proxy_settings(&self, new_settings: ProxySettings) {
        if let Ok(mut settings) = self.settings.lock() {
            *settings = new_settings;
            println!("[proxy] 代理设置已更新: {:?}", *settings);
        }
    }

    // 新增：获取当前代理设置
    pub fn get_proxy_settings(&self) -> ProxySettings {
        self.settings.lock().unwrap().clone()
    }
}

 