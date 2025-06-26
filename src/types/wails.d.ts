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