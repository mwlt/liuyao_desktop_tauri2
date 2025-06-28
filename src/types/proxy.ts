export interface SystemProxyInfo {
  http_proxy: string;
  https_proxy: string;
  socks_proxy: string;
  ftp_proxy: string;
  no_proxy: string;
  proxy_enabled: boolean;
}

export interface ProxyValidationResult {
  valid: boolean;
  message: string;
  formatted: string;
}

export interface TestResult {
  proxy_available: boolean;
  core333_accessible: boolean;
  google_accessible: boolean;
  message: string;
} 