/*
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-26 00:11:01
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-07-02 23:10:06
 * @FilePath: \liuyao_desktop_tauri\src-tauri\src\proxy_server.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
// 新增：文件操作和路径管理
use std::fs;
use std::path::{Path, PathBuf};

const BUFFER_SIZE: usize = 16 * 1024; // 16KB buffer，对大多数HTTP请求足够
const WS_BUFFER_SIZE: usize = 32 * 1024; // 32KB for WebSocket，为WebSocket连接提供更大缓冲区
const TIMEOUT: u64 = 10; // 60秒超时

// 新增：配置文件名称
const CONFIG_FILE_NAME: &str = "proxy_settings.json";

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
    pub direct_domains: Vec<String>, // 新增：直连域名列表
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    None,   // 不使用代理
    System, // 使用系统代理
    Http,   // HTTP代理
    Https,  // HTTPS代理
    Socks5, // SOCKS5代理
    Manual, // 手动配置（多种代理类型）
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
            direct_domains: vec![],
        }
    }
}

// 新增：获取配置文件路径
fn get_config_file_path() -> Result<PathBuf, String> {
    let app_data_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map_err(|_| "无法获取APPDATA环境变量".to_string())?
    } else {
        std::env::var("HOME")
            .map_err(|_| "无法获取HOME环境变量".to_string())?
    };
    
    let config_dir = Path::new(&app_data_dir).join("liuyao_desktop_tauri");
    
    // 确保配置目录存在
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    Ok(config_dir.join(CONFIG_FILE_NAME))
}

// 新增：保存配置到文件
pub fn save_settings_to_file(settings: &ProxySettings) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    
    let json_data = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    
    fs::write(&config_path, json_data)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    println!("[proxy] ✅ 配置已保存到: {:?}", config_path);
    Ok(())
}

// 新增：从文件加载配置
pub fn load_settings_from_file() -> Result<ProxySettings, String> {
    let config_path = get_config_file_path()?;
    
    if !config_path.exists() {
        println!("[proxy] 配置文件不存在，使用默认设置: {:?}", config_path);
        return Ok(ProxySettings::default());
    }
    
    let json_data = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;
    
    let settings: ProxySettings = serde_json::from_str(&json_data)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;
    
    println!("[proxy] ✅ 配置已从文件加载: {:?}", config_path);
    Ok(settings)
}

// 获取系统代理设置
#[derive(Debug)]
struct SystemProxyConfig {
    enabled: bool,
    http_proxy: Option<String>,
    https_proxy: Option<String>,
    bypass_list: Vec<String>,
    #[allow(dead_code)] // 允许未使用的字段
    pac_url: Option<String>, // 保留用于未来实现 PAC（代理自动配置）功能
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
                                    "http" => {
                                        config.http_proxy = Some(format!("http://{}", parts[1]))
                                    }
                                    "https" => {
                                        config.https_proxy = Some(format!("http://{}", parts[1]))
                                    }
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
                if let Ok(proxy_override) =
                    internet_settings.get_value::<String, _>("ProxyOverride")
                {
                    config.bypass_list = proxy_override.split(';').map(|s| s.to_string()).collect();
                }

                // 读取自动配置脚本URL
                if let Ok(auto_config_url) =
                    internet_settings.get_value::<String, _>("AutoConfigURL")
                {
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

// 智能分流：检查是否应该直连
fn should_direct_connect(target: &str, settings: &ProxySettings) -> bool {
    // 只在手动代理模式下检查 direct_domains
    if settings.proxy_type != ProxyType::Manual {
        // 只允许局域网/localhost 直连
        let host = extract_host(target);
        return is_local_address(&host);
    }

    // 手动代理模式下，才检查 direct_domains
    let host = extract_host(target);
    let host_lower = host.to_lowercase();

    for domain in &settings.direct_domains {
        let domain_lower = domain.to_lowercase();
        if host_lower == domain_lower || host_lower.ends_with(&format!(".{}", domain_lower)) {
            println!(
                "[proxy] 手动代理直连命中 - 域名: {}, 匹配规则: {}",
                host_lower, domain
            );
            return true;
        }
    }

    // 局域网/localhost 也允许直连
    is_local_address(&host_lower)
}

// 提取主机名的辅助函数
fn extract_host(target: &str) -> String {
    let host = if target.starts_with("http://") {
        &target[7..]
    } else if target.starts_with("https://") {
        &target[8..]
    } else {
        target
    };
    host.split(':')
        .next()
        .unwrap_or(host)
        .split('/')
        .next()
        .unwrap_or(host)
        .to_string()
}

// 检查是否是局域网地址
fn is_local_address(host: &str) -> bool {
    if host.starts_with("192.168.")
        || host.starts_with("10.")
        || (host.starts_with("172.") && host.len() > 4)
    {
        return true;
    }

    // 检查 172.16.0.0/12 范围
    if let Some(third_dot) = host.find('.').and_then(|i| host[i + 1..].find('.')) {
        if let Ok(second_octet) = host[4..4 + third_dot].parse::<u8>() {
            if host.starts_with("172.") && second_octet >= 16 && second_octet <= 31 {
                return true;
            }
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
            "无法解析地址",
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
        "所有地址连接失败",
    ))
}

// 新增：根据配置选择连接方式
fn connect_with_proxy_settings(
    target: &str,
    settings: &ProxySettings,
) -> std::io::Result<TcpStream> {
    // 首先检查代理是否启用
    if !settings.enabled {
        println!("[proxy] 代理已禁用，强制直连: {}", target);
        return direct_connect(target);
    }

    // 防止循环代理：如果目标是本地代理端口，直接连接
    if target.contains("127.0.0.1:8080") || target.contains("localhost:8080") {
        println!("[proxy] 检测到循环代理，改为直连: {}", target);
        return direct_connect(target);
    }

    // 智能分流：检查是否应该直连
    if should_direct_connect(target, settings) {
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
                    "系统代理未设置",
                ))
            }
        }
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
        }
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
        }
        ProxyType::Socks5 => {
            if let Some(proxy) = &settings.socks5_proxy {
                socks5_connect(target, proxy, &settings.username, &settings.password)
            } else {
                direct_connect(target)
            }
        }
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
            "HTTP proxy connection failed",
        ))
    }
}

// 新增：基础SOCKS5连接实现
fn socks5_connect(
    target: &str,
    proxy: &str,
    username: &Option<String>,
    password: &Option<String>,
) -> std::io::Result<TcpStream> {
    println!("[proxy] 通过SOCKS5代理连接: {} -> {}", target, proxy);

    let mut stream = TcpStream::connect(proxy)?;

    // SOCKS5握手
    socks5_handshake(&mut stream, username, password)?;

    // SOCKS5连接目标
    socks5_connect_target(&mut stream, target)?;

    Ok(stream)
}

// SOCKS5协议握手
fn socks5_handshake(
    stream: &mut TcpStream,
    username: &Option<String>,
    password: &Option<String>,
) -> std::io::Result<()> {
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
        0x00 => Ok(()),                                  // 无需认证
        0x02 => socks5_auth(stream, username, password), // 用户名密码认证
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Unsupported authentication method",
        )),
    }
}

// SOCKS5用户名密码认证
fn socks5_auth(
    stream: &mut TcpStream,
    username: &Option<String>,
    password: &Option<String>,
) -> std::io::Result<()> {
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
                "SOCKS5 authentication failed",
            ))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Missing credentials for SOCKS5 authentication",
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
            0x01 => {
                // IPv4
                let mut addr = [0u8; 6];
                stream.read_exact(&mut addr)?;
            }
            0x03 => {
                // 域名
                let mut len = [0u8; 1];
                stream.read_exact(&mut len)?;
                let mut addr = vec![0u8; len[0] as usize + 2];
                stream.read_exact(&mut addr)?;
            }
            0x04 => {
                // IPv6
                let mut addr = [0u8; 18];
                stream.read_exact(&mut addr)?;
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Invalid address type in response",
                ))
            }
        }

        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "SOCKS5 connection failed",
        ))
    }
}

// 辅助函数：解析目标地址
fn parse_target(target: &str) -> std::io::Result<(String, u16)> {
    if let Some(idx) = target.rfind(':') {
        let host = target[..idx].to_string();
        if let Ok(port) = target[idx + 1..].parse::<u16>() {
            Ok((host, port))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid port number",
            ))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Missing port in target address",
        ))
    }
}

fn tunnel(a: TcpStream, b: TcpStream) {
    let timeout = Duration::from_secs(TIMEOUT);
    let _ = a.set_read_timeout(Some(timeout));
    let _ = a.set_write_timeout(Some(timeout));
    let _ = b.set_read_timeout(Some(timeout));
    let _ = b.set_write_timeout(Some(timeout));

    // 设置TCP_NODELAY以优化性能
    let _ = a.set_nodelay(true);
    let _ = b.set_nodelay(true);

    // 克隆流以避免所有权问题
    let a2b = match a.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            println!("[proxy] 克隆流失败: {}", e);
            return;
        }
    };

    let b2a = match b.try_clone() {
        Ok(stream) => stream,
        Err(e) => {
            println!("[proxy] 克隆流失败: {}", e);
            return;
        }
    };

    // 使用Arc<AtomicBool>来控制线程终止
    let stop_signal = Arc::new(AtomicBool::new(false));
    let stop_signal_clone = Arc::clone(&stop_signal);

    // 根据连接类型选择缓冲区大小
    let buffer_size = if a2b
        .peer_addr()
        .map(|addr| addr.port() == 443)
        .unwrap_or(false)
    {
        WS_BUFFER_SIZE // WebSocket连接使用更大的缓冲区
    } else {
        BUFFER_SIZE // 普通HTTP连接使用标准缓冲区
    };

    // 创建两个线程处理双向数据传输
    let t1 = {
        let mut a2b = a2b;
        let mut b = b.try_clone().unwrap();
        let stop_signal = Arc::clone(&stop_signal);

        thread::spawn(move || {
            let mut buf = vec![0u8; buffer_size];
            while !stop_signal.load(Ordering::Relaxed) {
                match a2b.read(&mut buf) {
                    Ok(0) => break, // 连接关闭
                    Ok(n) => {
                        if b.write_all(&buf[..n]).is_err() || b.flush().is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            break;
                        }
                    }
                }
            }
            // 通知另一个线程也停止
            stop_signal.store(true, Ordering::Relaxed);
            // 显式释放缓冲区
            drop(buf);
        })
    };

    let t2 = {
        let mut b2a = b2a;
        let mut a = a.try_clone().unwrap();
        let stop_signal = stop_signal_clone;

        thread::spawn(move || {
            let mut buf = vec![0u8; buffer_size];
            while !stop_signal.load(Ordering::Relaxed) {
                match b2a.read(&mut buf) {
                    Ok(0) => break, // 连接关闭
                    Ok(n) => {
                        if a.write_all(&buf[..n]).is_err() || a.flush().is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() != std::io::ErrorKind::WouldBlock {
                            break;
                        }
                    }
                }
            }
            // 通知另一个线程也停止
            stop_signal.store(true, Ordering::Relaxed);
            // 显式释放缓冲区
            drop(buf);
        })
    };

    // 等待两个线程完成
    let _ = t1.join();
    let _ = t2.join();
}

fn handle_client(mut client_stream: TcpStream, settings: Arc<Mutex<ProxySettings>>) {
    // 设置TCP优化选项
    let _ = client_stream.set_nodelay(true);

    // 设置超时
    let timeout = Duration::from_secs(TIMEOUT);
    let _ = client_stream.set_read_timeout(Some(timeout));
    let _ = client_stream.set_write_timeout(Some(timeout));

    // 使用栈分配而不是堆分配来减少内存压力
    let mut buffer = [0u8; BUFFER_SIZE];
    let size = match client_stream.read(&mut buffer) {
        Ok(0) => return, // 连接已关闭
        Ok(n) => n,
        Err(e) => {
            println!("[proxy] 读取请求错误: {}", e);
            return;
        }
    };

    let request = match String::from_utf8_lossy(&buffer[..size]).to_string() {
        s if s.is_empty() => return,
        s => s,
    };

    let mut lines = request.lines();
    let request_line = match lines.next() {
        Some(line) => line,
        None => return,
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        println!("[proxy] 无效的请求行: {}", request_line);
        return;
    }

    println!("[proxy] 请求: {}", request_line);

    // 检查是否是WebSocket升级请求
    let is_websocket = request.to_lowercase().contains("upgrade: websocket");
    if is_websocket {
        println!("[proxy] 检测到WebSocket连接请求");
    }

    // 使用 Result 和 ? 操作符来简化错误处理
    if let Err(e) = handle_request(
        &mut client_stream,
        &parts,
        &request,
        is_websocket,
        &settings,
    ) {
        println!("[proxy] 处理请求失败: {}", e);
        let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
    }
}

// 将请求处理逻辑分离到单独的函数
fn handle_request(
    client_stream: &mut TcpStream,
    parts: &[&str],
    request: &str,
    is_websocket: bool,
    settings: &Arc<Mutex<ProxySettings>>,
) -> std::io::Result<()> {
    match parts[0].to_uppercase().as_str() {
        "CONNECT" => handle_connect_request(client_stream, parts, is_websocket, settings),
        _ => handle_http_request(client_stream, parts, request, is_websocket, settings),
    }
}

// 处理 CONNECT 请求
fn handle_connect_request(
    client_stream: &mut TcpStream,
    parts: &[&str],
    _is_websocket: bool, // 添加下划线前缀表示有意未使用
    settings: &Arc<Mutex<ProxySettings>>,
) -> std::io::Result<()> {
    let host_port = parts[1];
    let (host, port) = match host_port.find(':') {
        Some(idx) => {
            let host = &host_port[..idx];
            let port = host_port[idx + 1..].parse().unwrap_or(443);
            (host, port)
        }
        None => (host_port, 443),
    };

    let target_addr = format!("{}:{}", host, port);
    println!("[proxy] CONNECT请求到: {}", target_addr);

    if port == 8000 {
        println!("[proxy] 可能是WebSocket CONNECT请求");
    }

    let proxy_settings = settings.lock().unwrap().clone();

    match connect_with_proxy_settings(&target_addr, &proxy_settings) {
        Ok(target_stream) => {
            println!("[proxy] CONNECT隧道建立成功: {}", target_addr);
            client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")?;
            tunnel(client_stream.try_clone()?, target_stream);
            Ok(())
        }
        Err(e) => {
            println!("[proxy] CONNECT隧道建立失败: {} - {}", target_addr, e);
            client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
            Err(e)
        }
    }
}

// 处理 HTTP 请求
fn handle_http_request(
    client_stream: &mut TcpStream,
    parts: &[&str],
    request: &str,
    is_websocket: bool,
    settings: &Arc<Mutex<ProxySettings>>,
) -> std::io::Result<()> {
    let mut url = parts[1].to_string();

    if url.contains("/api/admin/") {
        url = url.replace("/api/admin/", "/admin/");
        println!("[proxy] URL重写: {} -> {}", parts[1], url);
    }

    if url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("ws://")
        || url.starts_with("wss://")
    {
        // 处理双斜杠问题：//www.core333.com//attachment -> //www.core333.com/attachment
        let clean_url = url
            .replace("//attachment", "/attachment")
            .replace("//static", "/static")
            .replace("//assets", "/assets")
            .replace("//uploads", "/uploads")
            .replace("//images", "/images")
            .replace("//css", "/css")
            .replace("//js", "/js");

        println!("[proxy] URL清理: {} -> {}", url, clean_url);
        handle_absolute_url(client_stream, &clean_url, request, is_websocket, settings)
    } else if url.starts_with("//") {
        // 处理协议相对路径中的双斜杠问题
        let clean_url = url
            .replace("//attachment", "/attachment")
            .replace("//static", "/static")
            .replace("//assets", "/assets")
            .replace("//uploads", "/uploads")
            .replace("//images", "/images")
            .replace("//css", "/css")
            .replace("//js", "/js");

        println!("[proxy] 协议相对路径URL清理: {} -> {}", url, clean_url);
        // 处理协议相对路径（Protocol-relative URL）
        handle_protocol_relative_url(client_stream, &clean_url, request, is_websocket, settings)
    } else if url.starts_with("/") {
        handle_relative_url(client_stream, &url, request, is_websocket)
    } else {
        println!("[proxy] 不支持的URL格式: {}", url);
        client_stream.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")?;
        Ok(())
    }
}

// 处理绝对URL请求
fn handle_absolute_url(
    client_stream: &mut TcpStream,
    url: &str,
    request: &str,
    is_websocket: bool,
    settings: &Arc<Mutex<ProxySettings>>,
) -> std::io::Result<()> {
    println!("[proxy] 处理绝对URL请求: {}", url);

    let is_https = url.starts_with("https://");
    let is_ws = url.starts_with("ws://");
    let is_wss = url.starts_with("wss://");
    let is_websocket = is_websocket || is_ws || is_wss;

    let url_without_scheme = &url[if is_https {
        8
    } else if is_wss {
        6
    } else if is_ws {
        5
    } else {
        7
    }..];
    let host_end = url_without_scheme
        .find('/')
        .unwrap_or(url_without_scheme.len());
    let host_port = &url_without_scheme[..host_end];

    let (host, port) = match host_port.find(':') {
        Some(idx) => {
            let host = &host_port[..idx];
            let port = host_port[idx + 1..]
                .parse()
                .unwrap_or(if is_https || is_wss {
                    443
                } else if is_ws {
                    8000
                } else {
                    80
                });
            (host, port)
        }
        None => (
            host_port,
            if is_https || is_wss {
                443
            } else if is_ws {
                8000
            } else {
                80
            },
        ),
    };

    let target_addr = format!("{}:{}", host, port);
    println!("[proxy] 目标地址: {}", target_addr);

    let proxy_settings = settings.lock().unwrap().clone();

    // 检查是否应该直连
    if should_direct_connect(url, &proxy_settings) {
        println!("[proxy] 使用直连方式访问: {}", url);
        match direct_connect(&target_addr) {
            Ok(mut target_stream) => {
                let timeout = if is_websocket {
                    Duration::from_secs(300)
                } else {
                    Duration::from_secs(TIMEOUT)
                };

                let _ = target_stream.set_read_timeout(Some(timeout));
                let _ = target_stream.set_write_timeout(Some(timeout));

                // 构建并发送修改后的请求
                let modified_request =
                    modify_request(request, url_without_scheme, host_end, is_websocket)?;
                println!(
                    "[proxy] 发送修改后的请求: {}",
                    modified_request.lines().next().unwrap_or("")
                );
                target_stream.write_all(modified_request.as_bytes())?;

                // 转发响应
                tunnel(client_stream.try_clone()?, target_stream);
                Ok(())
            }
            Err(e) => {
                println!("[proxy] 直连失败: {}", e);
                client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
                Err(e)
            }
        }
    } else {
        println!("[proxy] 使用代理方式访问: {}", url);
        match connect_with_proxy_settings(&target_addr, &proxy_settings) {
            Ok(mut target_stream) => {
                let timeout = if is_websocket {
                    Duration::from_secs(300)
                } else {
                    Duration::from_secs(TIMEOUT)
                };

                let _ = target_stream.set_read_timeout(Some(timeout));
                let _ = target_stream.set_write_timeout(Some(timeout));

                // 构建并发送修改后的请求
                let modified_request =
                    modify_request(request, url_without_scheme, host_end, is_websocket)?;
                println!(
                    "[proxy] 发送修改后的请求: {}",
                    modified_request.lines().next().unwrap_or("")
                );
                target_stream.write_all(modified_request.as_bytes())?;

                // 转发响应
                tunnel(client_stream.try_clone()?, target_stream);
                Ok(())
            }
            Err(e) => {
                println!("[proxy] 代理连接失败: {}", e);
                client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
                Err(e)
            }
        }
    }
}

// 处理协议相对路径URL请求（如 //www.core333.com/path）
fn handle_protocol_relative_url(
    client_stream: &mut TcpStream,
    url: &str,
    request: &str,
    is_websocket: bool,
    settings: &Arc<Mutex<ProxySettings>>,
) -> std::io::Result<()> {
    // 协议相对路径以 "//" 开头，需要根据当前请求的协议来决定使用 http 还是 https
    // 默认使用 HTTPS（大多数现代网站都支持 HTTPS）
    let mut use_https = true;

    // 检查请求头中的协议指示器
    for line in request.lines() {
        let line_lower = line.to_lowercase();
        if line_lower.starts_with("x-forwarded-proto:") && line_lower.contains("http:") {
            use_https = false;
            break;
        }
        // 检查 Host 头，如果是 80 端口通常表示 HTTP
        if line_lower.starts_with("host:") && line.contains(":80") {
            use_https = false;
            break;
        }
    }

    // 构建完整的URL
    let protocol = if use_https { "https" } else { "http" };
    let full_url = format!("{}{}", protocol, url);

    println!("[proxy] 协议相对路径 {} 转换为: {}", url, full_url);

    // 检查智能分流：从URL中提取主机名进行判断
    let proxy_settings = settings.lock().unwrap().clone();
    if should_direct_connect(&full_url, &proxy_settings) {
        println!("[proxy] 协议相对路径智能分流 - 直连: {}", full_url);
        // 直接连接处理
        let is_https = full_url.starts_with("https://");
        let url_without_scheme = &full_url[if is_https { 8 } else { 7 }..];
        let host_end = url_without_scheme
            .find('/')
            .unwrap_or(url_without_scheme.len());
        let host_port = &url_without_scheme[..host_end];

        let (host, port) = match host_port.find(':') {
            Some(idx) => {
                let host = &host_port[..idx];
                let port = host_port[idx + 1..]
                    .parse()
                    .unwrap_or(if is_https { 443 } else { 80 });
                (host, port)
            }
            None => (host_port, if is_https { 443 } else { 80 }),
        };

        let target_addr = format!("{}:{}", host, port);

        match direct_connect(&target_addr) {
            Ok(mut target_stream) => {
                let modified_request =
                    modify_request(request, url_without_scheme, host_end, is_websocket)?;
                target_stream.write_all(modified_request.as_bytes())?;
                tunnel(client_stream.try_clone()?, target_stream);
                Ok(())
            }
            Err(e) => {
                println!("[proxy] 直连失败: {}", e);
                client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
                Err(e)
            }
        }
    } else {
        // 使用现有的绝对URL处理函数（会通过代理）
        handle_absolute_url(client_stream, &full_url, request, is_websocket, settings)
    }
}

// 处理相对URL请求
fn handle_relative_url(
    client_stream: &mut TcpStream,
    url: &str,
    request: &str,
    _is_websocket: bool,
) -> std::io::Result<()> {
    let mut target_host = "localhost:1420";

    // 从请求头中获取Host
    for line in request.lines() {
        if line.to_lowercase().starts_with("host:") {
            if let Some(colon_pos) = line.find(':') {
                let host_value = line[colon_pos + 1..].trim();
                if !host_value.is_empty() && !host_value.contains(":8080") {
                    target_host = host_value;
                    break;
                }
            }
        }
    }

    println!("[proxy] 相对路径请求 {} 转发到: {}", url, target_host);

    match TcpStream::connect(target_host) {
        Ok(mut server_stream) => {
            let timeout = if _is_websocket {
                Duration::from_secs(300)
            } else {
                Duration::from_secs(TIMEOUT)
            };

            let _ = server_stream.set_read_timeout(Some(timeout));
            let _ = server_stream.set_write_timeout(Some(timeout));
            server_stream.write_all(request.as_bytes())?;

            tunnel(client_stream.try_clone()?, server_stream);
            Ok(())
        }
        Err(e) => {
            println!("[proxy] 连接失败: {} - {}", target_host, e);
            client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n")?;
            Err(e)
        }
    }
}

// 修改请求头
fn modify_request(
    request: &str,
    url_without_scheme: &str,
    host_end: usize,
    _is_websocket: bool, // 添加下划线前缀表示有意未使用
) -> std::io::Result<String> {
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

                if _is_websocket {
                    if let Some(slash_pos) = url_without_scheme[host_end..].find('/') {
                        path = url_without_scheme[host_end + slash_pos..].to_string();
                    } else {
                        path = "/".to_string();
                    }
                }

                modified_request.push_str(&format!(
                    "{} {} {}\r\n",
                    parts[0],
                    path,
                    parts.get(2).unwrap_or(&"HTTP/1.1")
                ));
            }
            is_first_line = false;
        } else {
            if _is_websocket
                || (!line.to_lowercase().starts_with("accept-encoding:")
                    && !line.to_lowercase().starts_with("proxy-connection:"))
            {
                modified_request.push_str(line);
                modified_request.push_str("\r\n");
            }
        }
    }
    modified_request.push_str("\r\n");

    Ok(modified_request)
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
                    let mut consecutive_errors = 0;
                    const MAX_ERRORS: u32 = 5;

                    for stream in listener.incoming() {
                        match stream {
                            Ok(client_stream) => {
                                consecutive_errors = 0; // 重置错误计数
                                let settings_clone = Arc::clone(&settings_clone);

                                // 限制最大并发连接数
                                if thread::available_parallelism()
                                    .map(|n| n.get() as u32)
                                    .unwrap_or(4)
                                    > 32
                                {
                                    println!("[proxy] 警告: 并发连接数过高");
                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                    continue;
                                }

                                thread::spawn(move || {
                                    let _ = client_stream.set_nodelay(true); // 优化网络性能
                                    handle_client(client_stream, settings_clone);
                                });
                            }
                            Err(e) => {
                                consecutive_errors += 1;
                                println!("[proxy] 接受连接错误: {}", e);

                                if consecutive_errors >= MAX_ERRORS {
                                    println!("[proxy] 连续错误过多，暂停接受新连接");
                                    std::thread::sleep(std::time::Duration::from_secs(1));
                                    consecutive_errors = 0;
                                }
                            }
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
            *settings = new_settings.clone();
            println!("[proxy] 代理设置已更新: {:?}", *settings);
            
            // 自动保存到文件
            if let Err(e) = save_settings_to_file(&new_settings) {
                println!("[proxy] ⚠️ 保存配置文件失败: {}", e);
            }
        }
    }

    // 新增：获取当前代理设置
    pub fn get_proxy_settings(&self) -> ProxySettings {
        self.settings.lock().unwrap().clone()
    }
}
