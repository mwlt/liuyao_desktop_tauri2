/*
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-26 00:11:01
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-26 21:05:23
 * @FilePath: \liuyao_desktop_tauri\src-tauri\src\proxy_server.rs
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use std::io::{Read, Write};
use std::time::Duration;

const BUFFER_SIZE: usize = 1024 * 1024; // 1MB buffer
const TIMEOUT: u64 = 60; // 60秒超时

// 获取系统HTTP代理设置
fn get_system_http_proxy() -> Option<String> {
    if let Ok(proxy) = std::env::var("http_proxy") {
        return Some(proxy);
    }
    if let Ok(proxy) = std::env::var("HTTP_PROXY") {
        return Some(proxy);
    }
    None
}

pub struct ProxyServer {
    pub port: u16,
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

fn handle_client(mut client_stream: TcpStream) {
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

        // 对于core333.com和相关域名强制直连
        let should_direct = host.contains("core333.com") || host.contains("qiniu.com") || 
                           host.contains("qiniucdn.com") || host.contains("file.core333.com");
        
        if should_direct {
            match direct_connect(&format!("{}:{}", host, port)) {
                Ok(target_stream) => {
                    let _ = client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n");
                    tunnel(client_stream, target_stream);
                }
                Err(_) => {
                    let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                }
            }
        } else {
            match get_system_http_proxy() {
                Some(proxy) => {
                    if let Ok(mut proxy_stream) = TcpStream::connect(&proxy) {
                        let connect_request = format!("CONNECT {} HTTP/1.1\r\nHost: {}\r\n\r\n", host_port, host_port);
                        let _ = proxy_stream.write_all(connect_request.as_bytes());
                        tunnel(client_stream, proxy_stream);
                    }
                }
                None => {
                    if let Ok(target_stream) = direct_connect(&format!("{}:{}", host, port)) {
                        let _ = client_stream.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n");
                        tunnel(client_stream, target_stream);
                    }
                }
            }
        }
        return;
    }
    
    // 处理HTTP请求
    let mut url = parts[1].to_string();
    
    // 重写API路径
    if url.contains("/api/admin/") {
        url = url.replace("/api/admin/", "/admin/");
        println!("[proxy] URL重写: {} -> {}", parts[1], url);
    }
    
    // 如果是绝对URL
    if url.starts_with("http://") {
        let url = &url[7..];
        let host_end = url.find('/').unwrap_or(url.len());
        let host_port = &url[..host_end];
        let mut host = host_port;
        let mut port = 80u16;
        if let Some(idx) = host_port.find(':') {
            host = &host_port[..idx];
            if let Ok(p) = host_port[idx+1..].parse() {
                port = p;
            }
        }
        
        let should_direct = host.contains("core333.com") || host.contains("qiniu.com") || 
                           host.contains("qiniucdn.com") || host.contains("file.core333.com");
        let target_addr = format!("{}:{}", host, port);
        
        let mut target_stream = if should_direct {
            match TcpStream::connect(&target_addr) {
                Ok(stream) => stream,
                Err(_) => {
                    let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                    return;
                }
            }
        } else {
            match get_system_http_proxy() {
                Some(proxy) => {
                    match TcpStream::connect(&proxy) {
                        Ok(stream) => stream,
                        Err(_) => {
                            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                            return;
                        }
                    }
                }
                None => {
                    match TcpStream::connect(&target_addr) {
                        Ok(stream) => stream,
                        Err(_) => {
                            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                            return;
                        }
                    }
                }
            }
        };

        let _ = target_stream.set_read_timeout(Some(timeout));
        let _ = target_stream.set_write_timeout(Some(timeout));

        // 构建新的请求
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
                    modified_request.push_str(&format!("{} {} {}\r\n", parts[0], path, parts[2]));
                }
                is_first_line = false;
            } else {
                if !line.to_lowercase().starts_with("accept-encoding:") &&
                   !line.to_lowercase().starts_with("connection:") &&
                   !line.to_lowercase().starts_with("proxy-connection:") {
                    modified_request.push_str(line);
                    modified_request.push_str("\r\n");
                }
            }
        }
        modified_request.push_str("\r\n");

        // 发送请求
        if target_stream.write_all(modified_request.as_bytes()).is_err() {
            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
            return;
        }

        // 转发响应
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
        
        if let Ok(mut server_stream) = TcpStream::connect(target_host) {
            let _ = server_stream.set_read_timeout(Some(timeout));
            let _ = server_stream.set_write_timeout(Some(timeout));
            let _ = server_stream.write_all(request.as_bytes());
            tunnel(client_stream, server_stream);
        } else {
            let _ = client_stream.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
        }
    }
}

impl ProxyServer {
    pub fn start_auto_port(range_start: u16, range_end: u16) -> Option<Self> {
        for port in range_start..=range_end {
            let addr = format!("127.0.0.1:{}", port);
            if let Ok(listener) = TcpListener::bind(&addr) {
                println!("[proxy] 监听端口: {}", port);
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        if let Ok(client_stream) = stream {
                            thread::spawn(move || handle_client(client_stream));
                        }
                    }
                });
                return Some(ProxyServer { port });
            }
        }
        None
    }
}

 