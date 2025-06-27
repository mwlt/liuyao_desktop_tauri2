use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemProxyInfo {
    pub http_proxy: String,
    pub https_proxy: String,
    pub socks_proxy: String,
    pub ftp_proxy: String,
    pub no_proxy: String,
    pub proxy_enabled: bool,
}

impl Default for SystemProxyInfo {
    fn default() -> Self {
        SystemProxyInfo {
            http_proxy: String::new(),
            https_proxy: String::new(),
            socks_proxy: String::new(),
            ftp_proxy: String::new(),
            no_proxy: String::new(),
            proxy_enabled: false,
        }
    }
}

#[cfg(target_os = "windows")]
pub fn get_system_proxy() -> SystemProxyInfo {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut proxy_info = SystemProxyInfo::default();
    
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(internet_settings) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings") {
        // 检查代理是否启用
        if let Ok(proxy_enable) = internet_settings.get_value::<u32, _>("ProxyEnable") {
            proxy_info.proxy_enabled = proxy_enable == 1;
        }
        
        // 读取代理服务器地址
        if let Ok(proxy_server) = internet_settings.get_value::<String, _>("ProxyServer") {
            // 解析代理服务器字符串
            if proxy_server.contains("=") {
                // 格式: "http=proxy:port;https=proxy:port;ftp=proxy:port;socks=proxy:port"
                for proxy in proxy_server.split(';') {
                    let parts: Vec<&str> = proxy.split('=').collect();
                    if parts.len() == 2 {
                        match parts[0] {
                            "http" => proxy_info.http_proxy = parts[1].to_string(),
                            "https" => proxy_info.https_proxy = parts[1].to_string(),
                            "ftp" => proxy_info.ftp_proxy = parts[1].to_string(),
                            "socks" => proxy_info.socks_proxy = parts[1].to_string(),
                            _ => {}
                        }
                    }
                }
            } else {
                // 所有协议使用相同的代理
                proxy_info.http_proxy = proxy_server.clone();
                proxy_info.https_proxy = proxy_server.clone();
                proxy_info.ftp_proxy = proxy_server.clone();
            }
        }
        
        // 读取不使用代理的地址
        if let Ok(proxy_override) = internet_settings.get_value::<String, _>("ProxyOverride") {
            proxy_info.no_proxy = proxy_override;
        }
    }
    
    proxy_info
}

#[cfg(target_os = "macos")]
pub fn get_system_proxy() -> SystemProxyInfo {
    use std::process::Command;
    
    let mut proxy_info = SystemProxyInfo::default();
    
    // 使用networksetup命令获取代理信息
    let services = Command::new("networksetup")
        .arg("-listallnetworkservices")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok());
    
    if let Some(services) = services {
        for service in services.lines().skip(1) {
            // 检查HTTP代理
            if let Ok(output) = Command::new("networksetup")
                .args(["-getwebproxy", service])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if output_str.contains("Enabled: Yes") {
                        proxy_info.proxy_enabled = true;
                        if let Some(proxy) = output_str.lines()
                            .find(|line| line.contains("Server:"))
                            .and_then(|line| line.split(": ").nth(1))
                        {
                            proxy_info.http_proxy = proxy.trim().to_string();
                        }
                    }
                }
            }
            
            // 检查HTTPS代理
            if let Ok(output) = Command::new("networksetup")
                .args(["-getsecurewebproxy", service])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(proxy) = output_str.lines()
                        .find(|line| line.contains("Server:"))
                        .and_then(|line| line.split(": ").nth(1))
                    {
                        proxy_info.https_proxy = proxy.trim().to_string();
                    }
                }
            }
            
            // 检查SOCKS代理
            if let Ok(output) = Command::new("networksetup")
                .args(["-getsocksfirewallproxy", service])
                .output()
            {
                if let Ok(output_str) = String::from_utf8(output.stdout) {
                    if let Some(proxy) = output_str.lines()
                        .find(|line| line.contains("Server:"))
                        .and_then(|line| line.split(": ").nth(1))
                    {
                        proxy_info.socks_proxy = proxy.trim().to_string();
                    }
                }
            }
        }
    }
    
    proxy_info
}

#[cfg(target_os = "linux")]
pub fn get_system_proxy() -> SystemProxyInfo {
    use std::env;
    let mut proxy_info = SystemProxyInfo::default();
    
    // 从环境变量读取代理设置
    if let Ok(http_proxy) = env::var("http_proxy").or_else(|_| env::var("HTTP_PROXY")) {
        proxy_info.http_proxy = http_proxy;
        proxy_info.proxy_enabled = true;
    }
    
    if let Ok(https_proxy) = env::var("https_proxy").or_else(|_| env::var("HTTPS_PROXY")) {
        proxy_info.https_proxy = https_proxy;
    }
    
    if let Ok(ftp_proxy) = env::var("ftp_proxy").or_else(|_| env::var("FTP_PROXY")) {
        proxy_info.ftp_proxy = ftp_proxy;
    }
    
    if let Ok(socks_proxy) = env::var("socks_proxy").or_else(|_| env::var("SOCKS_PROXY")) {
        proxy_info.socks_proxy = socks_proxy;
    }
    
    if let Ok(no_proxy) = env::var("no_proxy").or_else(|_| env::var("NO_PROXY")) {
        proxy_info.no_proxy = no_proxy;
    }
    
    // 对于Linux，还可以尝试读取GNOME设置
    if !proxy_info.proxy_enabled {
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.system.proxy", "mode"])
            .output()
        {
            if let Ok(mode) = String::from_utf8(output.stdout) {
                proxy_info.proxy_enabled = mode.contains("manual");
            }
        }
    }
    
    proxy_info
}

#[tauri::command]
pub fn get_system_proxy_info() -> SystemProxyInfo {
    get_system_proxy()
} 