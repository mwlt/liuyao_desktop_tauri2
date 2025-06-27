import { defineStore } from 'pinia';
import { ref, watch, computed, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ProxySettings } from '../types/wails';

export type ProxyType = 'none' | 'system' | 'manual';

// 简化的前端代理配置
interface ProxyConfig {
  type: ProxyType;
  httpProxy: string;
  httpsProxy: string;
  socksProxy: string;
}

// 默认配置
const defaultConfig: ProxyConfig = {
  type: 'none',
  httpProxy: '',
  httpsProxy: '',
  socksProxy: ''
};

interface TestResult {
  proxy_available: boolean;
  core333_accessible: boolean;
  google_accessible: boolean;
  message: string;
}

export const useProxyStore = defineStore('proxy', () => {
  const config = ref<ProxyConfig>({ ...defaultConfig });
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isInitialized = ref(false);

  // 本地代理端口
  const localProxyPort = ref<number | null>(null);
  
  // 当前代理设置（来自Rust后端）
  const currentProxySettings = ref<ProxySettings | null>(null);

  // 获取本地代理端口
  async function initLocalProxyPort(retry = 3) {
    console.log(`[ProxyStore] 获取本地代理端口，剩余重试: ${retry}`);
    
    try {
      const port = await invoke<number>('get_local_proxy_port');
      console.log('[ProxyStore] 获取端口结果:', port);
      
      if (port && port !== 0) {
        localProxyPort.value = port;
        console.log('[ProxyStore] ✅ 本地代理端口:', port);
        return;
      } else {
        console.log('[ProxyStore] ⚠️ 端口无效:', port);
        if (retry > 0) {
          setTimeout(() => initLocalProxyPort(retry - 1), 500);
        } else {
          localProxyPort.value = null;
        }
      }
    } catch (e) {
      console.error('[ProxyStore] ❌ 获取端口失败:', e);
      if (retry > 0) {
        setTimeout(() => initLocalProxyPort(retry - 1), 500);
      } else {
        localProxyPort.value = null;
      }
    }
  }

  // 从后端加载代理设置
  async function loadProxySettings() {
    try {
      isLoading.value = true;
      error.value = null;
      
      const settings = await invoke<ProxySettings>('get_proxy_settings');
      currentProxySettings.value = settings;
      
      // 同步到前端配置
      syncFromBackendSettings(settings);
      
      console.log('[ProxyStore] ✅ 加载代理设置成功:', settings);
    } catch (e) {
      console.error('[ProxyStore] ❌ 加载代理设置失败:', e);
      error.value = `加载代理设置失败: ${e}`;
    } finally {
      isLoading.value = false;
    }
  }

  // 将后端设置同步到前端配置
  function syncFromBackendSettings(settings: ProxySettings) {
    // 映射代理类型
    const typeMap: Record<string, ProxyType> = {
      'None': 'none',
      'System': 'system', 
      'Manual': 'manual',
      'Http': 'manual',
      'Https': 'manual',
      'Socks5': 'manual'
    };
    
    config.value.type = typeMap[settings.proxy_type] || 'none';
    config.value.httpProxy = settings.http_proxy || '';
    config.value.httpsProxy = settings.https_proxy || '';
    config.value.socksProxy = settings.socks5_proxy || '';
    
    console.log('[ProxyStore] 同步后端设置到前端:', config.value);
  }

  // 更新后端代理设置
  async function updateBackendProxySettings() {
    try {
      if (!currentProxySettings.value) {
        console.warn('[ProxyStore] 没有当前代理设置，跳过更新');
        return;
      }
      
      isLoading.value = true;
      await invoke('update_proxy_settings', { settings: currentProxySettings.value });
      console.log('[ProxyStore] ✅ 后端代理设置更新成功');
    } catch (e) {
      console.error('[ProxyStore] ❌ 更新后端代理设置失败:', e);
      error.value = `更新后端代理设置失败: ${e}`;
    } finally {
      isLoading.value = false;
    }
  }

  // 计算属性：检查是否有任何代理配置
  const hasProxyConfig = computed(() => {
    return config.value.httpProxy !== '' || 
           config.value.httpsProxy !== '' || 
           config.value.socksProxy !== '';
  });

  // 监听配置变化，同步到后端
  watch(
    config,
    async (newConfig) => {
      if (!isInitialized.value) {
        return;
      }

      console.log('[ProxyStore] 代理配置变化:', newConfig);

      // 更新当前代理设置
      if (currentProxySettings.value) {
        // 映射前端类型到后端类型
        const backendTypeMap: Record<ProxyType, string> = {
          'none': 'None',
          'system': 'System',
          'manual': 'Manual'
        };
        
        currentProxySettings.value.proxy_type = backendTypeMap[newConfig.type] as any;
        currentProxySettings.value.http_proxy = newConfig.httpProxy || undefined;
        currentProxySettings.value.https_proxy = newConfig.httpsProxy || undefined;
        currentProxySettings.value.socks5_proxy = newConfig.socksProxy || undefined;
        
        // 更新后端
        await updateBackendProxySettings();
      }
    },
    { deep: true }
  );

  // 设置代理类型
  async function setType(type: ProxyType) {
    config.value.type = type;
    
    try {
      const backendTypeMap: Record<ProxyType, string> = {
        'none': 'None',
        'system': 'System',
        'manual': 'Manual'
      };
      
      await invoke('set_proxy_type', { proxyType: backendTypeMap[type] });
      console.log('[ProxyStore] ✅ 设置代理类型成功:', type);
      
      // 如果是禁用代理，清除所有代理地址
      if (type === 'none') {
        await clearAll();
      }
      // 如果是系统代理，自动应用系统代理设置
      else if (type === 'system') {
        const result = await applySystemProxy();
        if (!result.success) {
          error.value = result.message;
        }
      }
      
      // 打印当前代理状态
      await getProxyStatus();
    } catch (e) {
      console.error('[ProxyStore] ❌ 设置代理类型失败:', e);
      error.value = `设置代理类型失败: ${e}`;
      throw e; // 重新抛出错误，让调用者知道发生了错误
    }
  }

  // 设置HTTP代理
  async function setHTTPProxy(addr: string) {
    config.value.httpProxy = addr;
    
    try {
      await invoke('set_http_proxy', { proxy: addr });
      console.log('[ProxyStore] ✅ 设置HTTP代理成功:', addr);
    } catch (e) {
      console.error('[ProxyStore] ❌ 设置HTTP代理失败:', e);
      error.value = `设置HTTP代理失败: ${e}`;
    }
  }

  // 设置HTTPS代理
  async function setHTTPSProxy(addr: string) {
    config.value.httpsProxy = addr;
    
    try {
      await invoke('set_https_proxy', { proxy: addr });
      console.log('[ProxyStore] ✅ 设置HTTPS代理成功:', addr);
    } catch (e) {
      console.error('[ProxyStore] ❌ 设置HTTPS代理失败:', e);
      error.value = `设置HTTPS代理失败: ${e}`;
    }
  }

  // 设置SOCKS代理
  async function setSOCKSProxy(addr: string) {
    config.value.socksProxy = addr;
    
    try {
      await invoke('set_socks5_proxy', { proxy: addr });
      console.log('[ProxyStore] ✅ 设置SOCKS5代理成功:', addr);
    } catch (e) {
      console.error('[ProxyStore] ❌ 设置SOCKS5代理失败:', e);
      error.value = `设置SOCKS5代理失败: ${e}`;
    }
  }

  // 简化的代理验证（基础格式检查）
  function validateProxyAddress(address: string): { valid: boolean; message: string; formatted: string } {
    if (!address.trim()) {
      return { valid: false, message: '代理地址不能为空', formatted: '' };
    }

    // 基础格式检查
    const patterns = [
      /^https?:\/\/[\w.-]+:\d+$/,  // http://host:port
      /^socks5:\/\/[\w.-]+:\d+$/,  // socks5://host:port
      /^[\w.-]+:\d+$/             // host:port
    ];

    const isValid = patterns.some(pattern => pattern.test(address));
    
    if (!isValid) {
      return { 
        valid: false, 
        message: '代理地址格式不正确，应为 host:port 或 protocol://host:port', 
        formatted: '' 
      };
    }

    // 格式化地址
    let formatted = address;
    if (!/^https?:\/\/|^socks5:\/\//.test(address)) {
      formatted = `http://${address}`;
    }

    return { valid: true, message: '代理地址格式正确', formatted };
    }
    
  // 更新：测试代理连接
  async function testProxyConnectivity(proxyUrl: string): Promise<{ success: boolean; message: string; details?: TestResult }> {
    try {
      console.log('[ProxyStore] 开始测试代理连接:', proxyUrl);
      
      const result = await invoke<TestResult>('test_proxy_connectivity', { proxyUrl });
      
      if (result.proxy_available) {
        console.log('[ProxyStore] ✅ 代理连接测试结果:', result);
        return { 
          success: true, 
          message: '代理连接测试完成', 
          details: result 
        };
      } else {
        console.log('[ProxyStore] ❌ 代理连接测试失败:', result);
        return { 
          success: false, 
          message: result.message,
          details: result 
        };
      }
    } catch (e) {
      console.error('[ProxyStore] ❌ 代理连接测试异常:', e);
      return { 
        success: false, 
        message: `测试失败: ${e}` 
      };
    }
  }

  // 新增：应用系统代理设置
  async function applySystemProxy(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('[ProxyStore] 开始应用系统代理设置');
      
      await invoke('apply_system_proxy');
      
      // 重新加载设置以同步前端状态
      await loadProxySettings();
      
      console.log('[ProxyStore] ✅ 系统代理设置应用成功');
      return { success: true, message: '系统代理设置应用成功' };
    } catch (e) {
      console.error('[ProxyStore] ❌ 应用系统代理设置失败:', e);
      return { success: false, message: `应用失败: ${e}` };
    }
  }
  
  // 新增：应用手动代理设置
  async function applyManualProxy(): Promise<{ success: boolean; message: string }> {
    try {
      console.log('[ProxyStore] 开始应用手动代理设置');
      
      await invoke('apply_manual_proxy');
      
      // 重新加载设置以同步前端状态
      await loadProxySettings();
      
      console.log('[ProxyStore] ✅ 手动代理设置应用成功');
      return { success: true, message: '手动代理设置应用成功' };
    } catch (e) {
      console.error('[ProxyStore] ❌ 应用手动代理设置失败:', e);
      return { success: false, message: `应用失败: ${e}` };
    }
  }


  // 新增：获取代理状态
  async function getProxyStatus(): Promise<string> {
    try {
      const status = await invoke<string>('get_proxy_status');
      console.log('[ProxyStore] 当前代理状态:', status);
      return status;
    } catch (e) {
      console.error('[ProxyStore] ❌ 获取代理状态失败:', e);
      return `获取状态失败: ${e}`;
    }
  }

  // 应用到全部功能（增强版，包含代理测试）
  async function applyToAll(sourceAddress: string): Promise<boolean> {
    error.value = null;
    
    try {
      const validationResult = validateProxyAddress(sourceAddress);
      if (!validationResult.valid) {
        error.value = validationResult.message;
        return false;
      }

      const formattedAddress = validationResult.formatted;
      
      // 先测试代理连接
      console.log('[ProxyStore] 测试代理连接...');
      const testResult = await testProxyConnectivity(formattedAddress);
      if (!testResult.success) {
        error.value = `代理不可用: ${testResult.message}`;
        return false;
      }
      
      // 提取host:port部分
      const match = formattedAddress.match(/^(?:https?|socks5):\/\/(.*)$/);
      const hostAndPort = match ? match[1] : formattedAddress;

      // 为每种协议设置代理
      await setHTTPProxy(`http://${hostAndPort}`);
      await setHTTPSProxy(`https://${hostAndPort}`);
      await setSOCKSProxy(`socks5://${hostAndPort}`);

      // 打印当前代理状态
      await getProxyStatus();

      console.log('[ProxyStore] ✅ 应用到全部代理成功');
      return true;
    } catch (e: any) {
      error.value = `应用时发生错误: ${e instanceof Error ? e.message : String(e)}`;
      console.error('[ProxyStore] ❌ 应用到全部失败:', e);
      return false;
    }
  }

  // 清除所有代理配置
  async function clearAll() {
    try {
      await setHTTPProxy('');
      await setHTTPSProxy('');
      await setSOCKSProxy('');
      console.log('[ProxyStore] ✅ 清除所有代理配置成功');
    } catch (e) {
      console.error('[ProxyStore] ❌ 清除代理配置失败:', e);
      error.value = `清除代理配置失败: ${e}`;
    }
  }

  // 重置配置为初始状态
  async function resetConfig() {
    try {
      await setType('none');
      await clearAll();
      console.log('[ProxyStore] ✅ 重置配置成功');
    } catch (e) {
      console.error('[ProxyStore] ❌ 重置配置失败:', e);
      error.value = `重置配置失败: ${e}`;
    }
  }

  // 初始化
  async function initialize() {
    try {
      console.log('[ProxyStore] 开始初始化...');
      
      // 并行初始化
      await Promise.all([
        initLocalProxyPort(),
        loadProxySettings()
      ]);
      
      isInitialized.value = true;
      console.log('[ProxyStore] ✅ 初始化完成');
    } catch (e) {
      console.error('[ProxyStore] ❌ 初始化失败:', e);
      error.value = `初始化失败: ${e}`;
    }
  }

  return { 
    // 状态
    config: readonly(config),
    isLoading: readonly(isLoading),
    error: readonly(error),
    isInitialized: readonly(isInitialized),
    localProxyPort: readonly(localProxyPort),
    currentProxySettings: readonly(currentProxySettings),
    
    // 计算属性
    hasProxyConfig,
    
    // 方法
    setType, 
    setHTTPProxy,
    setHTTPSProxy,
    setSOCKSProxy,
    validateProxyAddress,
    applyToAll,
    clearAll,
    resetConfig,
    initialize,
    loadProxySettings,
    updateBackendProxySettings,
    // 新增方法
    testProxyConnectivity,
    applySystemProxy,
    applyManualProxy,
    getProxyStatus,
  };
}); 