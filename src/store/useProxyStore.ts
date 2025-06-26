import { defineStore } from 'pinia';
import { ref, watch, computed, readonly } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ProxyConfig, ProxyValidationResult } from '../types/wails';

export type ProxyType = 'none' | 'system' | 'manual';

// 默认配置
const defaultConfig: ProxyConfig = {
  type: 'none',
  address: '',
  httpProxy: '',
  httpsProxy: '',
  socksProxy: '',
  noProxy: ''
};

export const useProxyStore = defineStore('proxy', () => {
  const config = ref<ProxyConfig>({ ...defaultConfig });
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isInitialized = ref(false);
  const isSyncing = ref(false);

  // 新增：本地代理端口
  const localProxyPort = ref<number | null>(null);
  async function initLocalProxyPort(retry = 3) {
    console.log(`initLocalProxyPort: 开始尝试获取端口，剩余重试次数: ${retry}`);
    
    try {
      console.log('正在调用 Tauri invoke get_local_proxy_port...');
      const port = await invoke<number>('get_local_proxy_port');
      console.log('get_local_proxy_port 返回结果:', port, '类型:', typeof port);
      
      if (port && port !== 0) {
        localProxyPort.value = port;
        console.log('✅ 成功获取本地代理端口:', port);
        return; // 成功获取，直接返回
      } else {
        console.log('⚠️ 获取到的端口无效:', port);
        if (retry > 0) {
          console.log(`将在500ms后重试，剩余重试次数: ${retry - 1}`);
          setTimeout(() => initLocalProxyPort(retry - 1), 500);
        } else {
          console.log('❌ 重试次数已用完，设置端口为null');
          localProxyPort.value = null;
        }
      }
    } catch (e) {
      console.error('❌ get_local_proxy_port 调用错误:', e);
      if (retry > 0) {
        console.log(`将在500ms后重试，剩余重试次数: ${retry - 1}`);
        setTimeout(() => initLocalProxyPort(retry - 1), 500);
      } else {
        console.log('❌ 重试次数已用完，设置端口为null');
        localProxyPort.value = null;
      }
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
    async (newConfig, oldConfig) => {
      if (!isInitialized.value || isSyncing.value) {
        return;
      }

      console.log('代理配置变化:', { old: oldConfig, new: newConfig });

      // 同步到后端
      if (window.go && window.go.main && window.go.main.App && window.go.main.App.UpdateProxyConfig) {
        try {
          isLoading.value = true;
          error.value = null;
          isSyncing.value = true;
          
          await window.go.main.App.UpdateProxyConfig(newConfig);
          console.log('代理配置已同步到后端:', newConfig);
        } catch (err) {
          error.value = `配置同步失败: ${err}`;
          console.error('代理配置同步失败:', err);
        } finally {
          isLoading.value = false;
          setTimeout(() => {
            isSyncing.value = false;
          }, 100);
        }
      }
    },
    { deep: true }
  );

  // 设置代理类型
  function setType(type: ProxyType) {
    if (isSyncing.value) return;
    config.value.type = type;
    // 当切换到非手动模式时，不再清除手动配置，以便用户可以轻松切换回来
    // if (type !== 'manual') {
    //   config.value.httpProxy = '';
    //   config.value.httpsProxy = '';
    //   config.value.socksProxy = '';
    //   config.value.noProxy = '';
    //   config.value.address = '';
    // }
  }

  // 设置HTTP代理
  function setHTTPProxy(addr: string) {
    if (isSyncing.value) return;
    config.value.httpProxy = addr;
    // 保持address字段同步（向后兼容）
    if (addr) config.value.address = addr;
  }

  // 设置HTTPS代理
  function setHTTPSProxy(addr: string) {
    if (isSyncing.value) return;
    config.value.httpsProxy = addr;
  }

  // 设置SOCKS代理
  function setSOCKSProxy(addr: string) {
    if (isSyncing.value) return;
    config.value.socksProxy = addr;
  }

  // 设置代理例外
  function setNoProxy(noProxy: string) {
    if (isSyncing.value) return;
    config.value.noProxy = noProxy;
  }

  // 校验代理地址
  async function validateProxy(address: string): Promise<ProxyValidationResult> {
    if (!window.go?.main?.App?.ValidateProxyAddress) {
      return { valid: false, message: '校验功能不可用', formatted: '' };
    }
    
    try {
      return await window.go.main.App.ValidateProxyAddress(address);
    } catch (err) {
      return { valid: false, message: `校验失败: ${err}`, formatted: '' };
    }
  }

  // 应用到全部功能
  async function applyToAll(sourceAddress: string): Promise<boolean> {
    error.value = null;
    try {
      const validationResult = await validateProxy(sourceAddress);
      if (!validationResult.valid) {
        error.value = validationResult.message;
        return false;
      }

      const formattedAddress = validationResult.formatted;
      
      // 使用正则表达式从格式化后的地址中提取 host 和 port
      const match = formattedAddress.match(/^(?:https?|socks5):\/\/(.*)$/);
      const hostAndPort = match ? match[1] : formattedAddress;

      // 为每种协议应用正确的前缀
      config.value.httpProxy = `http://${hostAndPort}`;
      config.value.httpsProxy = `https://${hostAndPort}`;
      config.value.socksProxy = `socks5://${hostAndPort}`;
      config.value.address = `http://${hostAndPort}`; // 兼容旧版

      return true;
    } catch (e: any) {
      error.value = `应用时发生错误: ${e instanceof Error ? e.message : String(e)}`;
      return false;
    }
  }

  // 清除所有代理配置
  function clearAll() {
    config.value.address = '';
    config.value.httpProxy = '';
    config.value.httpsProxy = '';
    config.value.socksProxy = '';
    config.value.noProxy = '';
  }

  // 重置配置为初始状态
  function resetConfig() {
    config.value.type = 'none';
    clearAll();
  }

  // 从后端初始化配置
  async function initFromBackend() {
    if (window.go && window.go.main && window.go.main.App && window.go.main.App.GetProxyConfig) {
      try {
        isLoading.value = true;
        isSyncing.value = true;
        
        const backendConfig = await window.go.main.App.GetProxyConfig();
        console.log('从后端获取配置:', backendConfig);
        
        // 合并配置，确保所有字段都存在
        config.value = {
          ...defaultConfig,
          ...backendConfig
        };
        
        isInitialized.value = true;
        console.log('代理配置初始化完成:', config.value);
      } catch (err) {
        error.value = `配置初始化失败: ${err}`;
        console.error('代理配置初始化失败:', err);
        isInitialized.value = true;
      } finally {
        isLoading.value = false;
        setTimeout(() => {
          isSyncing.value = false;
        }, 200);
      }
    } else {
      isInitialized.value = true;
    }
  }

  // 监听后端配置更新事件
  function listenToBackendEvents() {
    if (window.runtime && window.runtime.EventsOn) {
      window.runtime.EventsOn('proxy-config-updated', (newConfig: ProxyConfig) => {
        console.log('收到后端代理配置更新:', newConfig);
        isSyncing.value = true;
        config.value = { ...defaultConfig, ...newConfig };
        setTimeout(() => {
          isSyncing.value = false;
        }, 100);
      });
    }
  }

  // 清理事件监听
  function cleanup() {
    if (window.runtime && window.runtime.EventsOff) {
      window.runtime.EventsOff('proxy-config-updated');
    }
  }

  return { 
    config,
    isLoading,
    error,
    isInitialized,
    hasProxyConfig,
    setType, 
    setHTTPProxy,
    setHTTPSProxy,
    setSOCKSProxy,
    setNoProxy,
    validateProxy,
    applyToAll,
    clearAll,
    resetConfig,
    initFromBackend,
    listenToBackendEvents,
    cleanup,
    localProxyPort,
    initLocalProxyPort
  };
}, {
  persist: {
    key: 'proxy-config',
    storage: localStorage,
 
  }
}); 