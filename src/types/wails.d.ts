// Wails API 类型定义

export interface NetworkStatus {
  isOnline: boolean
  canReachTarget: boolean
  responseTime: number
  errorMessage: string
  lastCheck: string
  checkCount?: number
}

export interface ProxyConfig {
  type: 'none' | 'system' | 'manual'
  address: string        // 兼容旧版本的单一代理地址
  httpProxy: string     // HTTP代理地址
  httpsProxy: string    // HTTPS代理地址
  socksProxy: string    // SOCKS5代理地址
  noProxy: string       // 代理例外列表
}

// 新增：代理设置类型（对应Rust的ProxySettings）
export interface ProxySettings {
  proxy_type: ProxyType
  http_proxy?: string
  https_proxy?: string
  socks5_proxy?: string
  username?: string
  password?: string
  enabled: boolean
}

// 新增：代理类型枚举
export type ProxyType = 'None' | 'System' | 'Http' | 'Https' | 'Socks5' | 'Manual'

export interface ProxyValidationResult {
  valid: boolean
  message: string
  formatted: string
}

export interface LegacyProxyConfig {
  enabled: boolean
  type: string
  host: string
  port: string
}

// 新增：系统代理信息接口
export interface SystemProxyInfo {
  http_proxy: string
  https_proxy: string
  socks_proxy: string
  ftp_proxy: string
  no_proxy: string
  proxy_enabled: boolean
}

// Extend Window interface for Wails
declare global {
  interface Window {
    go: {
      main: {
        App: {
          // 网络相关方法
          TestNetwork: () => Promise<NetworkStatus>
          GetNetworkStatus: () => Promise<NetworkStatus>
          OpenInBrowser: (url: string) => Promise<void>
          RefreshPage: () => Promise<void>
          
          // 代理相关方法
          GetProxyConfig: () => Promise<ProxyConfig>
          SetProxyConfig: (config: ProxyConfig) => Promise<void>
          UpdateProxyConfig: (config: ProxyConfig) => Promise<void>
          ToggleSystemProxy: () => Promise<LegacyProxyConfig>
          ValidateProxyAddress: (address: string) => Promise<ProxyValidationResult>
          
          // 本地代理服务器
          GetLocalProxyURL: () => Promise<string>
          
          // 系统代理信息
          GetSystemProxyInfo: () => Promise<any>
          GetActiveProxyInfo: () => Promise<any>
        }
      }
    }
    runtime: {
      EventsOn: (eventName: string, callback: (data: any) => void) => void
      EventsOff: (eventName: string) => void
    }
  }
}

// 新增：Tauri API类型定义
declare global {
  const __TAURI__: {
    invoke: <T = any>(cmd: string, args?: Record<string, any>) => Promise<T>
  }
}

// 新增：Tauri命令类型定义
export interface TauriCommands {
  // 系统代理相关
  get_system_proxy_info: () => Promise<SystemProxyInfo>
  get_local_proxy_port: () => Promise<number>
  
  // 代理设置管理
  update_proxy_settings: (settings: ProxySettings) => Promise<void>
  get_proxy_settings: () => Promise<ProxySettings>
  set_proxy_type: (proxy_type: string) => Promise<void>
  set_http_proxy: (proxy: string) => Promise<void>
  set_https_proxy: (proxy: string) => Promise<void>
  set_socks5_proxy: (proxy: string) => Promise<void>
} 