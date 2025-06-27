<template>
  <div class="proxy-settings-popover">
    <!-- <div v-if="resultMessage" :class="['result-message', `result-message--${resultType}`]">
      {{ resultMessage }}
    </div> -->
    
    <n-radio-group v-model:value="proxyType" name="proxy-type-group">
      <n-space>
        <n-radio value="none">
          <span class="radio-label">禁用代理</span>
        </n-radio>
        <n-radio value="system">
          <span class="radio-label">系统代理</span>
        </n-radio>
        <n-radio value="manual">
          <span class="radio-label">手动代理</span>
        </n-radio>
      </n-space>
    </n-radio-group>

    <!-- 手动代理配置 -->
    <div v-if="proxyType === 'manual'" class="manual-proxy-section">
      <!-- HTTP代理 -->
      <div class="form-item">
        <label>HTTP代理</label>
        <div class="input-with-button">
          <n-input
            v-model:value="httpProxy"
            placeholder="如: 127.0.0.1:8888"
            clearable
            :status="httpProxyValidation.status"
            @input="debouncedValidateHttpProxy(httpProxy)"
          />
          <n-button 
            size="small" 
            @click="applyToAll(httpProxy, 'http')"
            :disabled="!httpProxy || httpProxyValidation.status === 'error'"
            quaternary
          >
            应用到全部
          </n-button>
        </div>
        <div v-if="httpProxyValidation.message" class="validation-message" :class="httpProxyValidation.status">
          {{ httpProxyValidation.message }}
        </div>
      </div>

      <!-- HTTPS代理 -->
      <div class="form-item">
        <label>HTTPS代理</label>
        <div class="input-with-button">
          <n-input
            v-model:value="httpsProxy"
            placeholder="如: 127.0.0.1:8888"
            clearable
            :status="httpsProxyValidation.status"
            @input="debouncedValidateHttpsProxy(httpsProxy)"
          />
          <n-button 
            size="small" 
            @click="applyToAll(httpsProxy, 'https')"
            :disabled="!httpsProxy || httpsProxyValidation.status === 'error'"
            quaternary
          >
            应用到全部
          </n-button>
        </div>
        <div v-if="httpsProxyValidation.message" class="validation-message" :class="httpsProxyValidation.status">
          {{ httpsProxyValidation.message }}
        </div>
      </div>

      <!-- SOCKS5代理 -->
      <div class="form-item">
        <label>SOCKS5代理</label>
        <div class="input-with-button">
          <n-input
            v-model:value="socksProxy"
            placeholder="如: socks5://127.0.0.1:1080"
            clearable
            :status="socksProxyValidation.status"
            @input="debouncedValidateSocksProxy(socksProxy)"
          />
          <n-button 
            size="small" 
            @click="applyToAll(socksProxy, 'socks')"
            :disabled="!socksProxy || socksProxyValidation.status === 'error'"
            quaternary
          >
            应用到全部
          </n-button>
        </div>
        <div v-if="socksProxyValidation.message" class="validation-message" :class="socksProxyValidation.status">
          {{ socksProxyValidation.message }}
        </div>
        <div class="proxy-hint">
          <small>注意：SOCKS5代理需要特殊处理，可能需要额外配置</small>
        </div>
      </div>

      <!-- 代理例外 -->
      <div class="form-item">
        <label>代理例外</label>
        <n-input
          v-model:value="noProxy"
          placeholder="如: localhost,127.0.0.1,*.local"
          clearable
        />
        <div class="proxy-hint">
          <small>多个地址用逗号分隔，支持通配符</small>
        </div>
      </div>

      <!-- 快速操作 -->
      <div class="quick-actions">

        <n-button size="small" @click="toggleProxy" :loading="loading" :type="proxyEnabled ? 'warning' : 'primary'">
          {{ proxyEnabled ? '禁用代理' : '启用代理' }}
        </n-button>
        <n-button size="small" @click="clearAllProxies" quaternary>
          清除全部
        </n-button>
      </div>
    </div>

    <!-- 系统代理信息显示 -->
    <div v-if="proxyType === 'system'" class="system-proxy-section">
      <hr style="margin: 16px 0; border: none; border-top: 1px solid #eee;" />
      <div class="system-proxy-header">
        <h4>系统代理信息</h4>
        <div class="system-proxy-actions">
        <n-button size="tiny" @click="refreshSystemProxy" :loading="loadingSystemProxy" quaternary>
          刷新
        </n-button>
          <!-- <n-button 
            size="tiny" 
            @click="applySystemProxySettings" 
            :loading="loadingSystemProxy"
            :disabled="!systemProxyInfo.proxy_enabled"
            type="primary"
          >
            应用
          </n-button> -->
        </div>
      </div>
      
      <div v-if="systemProxyInfo" class="system-proxy-info">
        <div class="proxy-info-item">
          <span class="proxy-label">HTTP代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.http_proxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">HTTPS代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.https_proxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">SOCKS代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.socks_proxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">FTP代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.ftp_proxy }}</span>
        </div>
        <div v-if="systemProxyInfo.no_proxy" class="proxy-info-item">
          <span class="proxy-label">代理例外:</span>
          <span class="proxy-value no-proxy">{{ systemProxyInfo.no_proxy }}</span>
        </div>
        <div class="proxy-status-item">
          <span class="proxy-label">状态:</span>
          <span class="proxy-status" :class="{ enabled: systemProxyInfo.proxy_enabled }">
            {{ systemProxyInfo.proxy_enabled ? '已启用' : '未启用' }}
          </span>
        </div>
      </div>
      
      <div v-else-if="loadingSystemProxy" class="loading-info">
        <span>正在获取系统代理信息...</span>
      </div>
      
      <div v-else class="no-proxy-info">
        <span>无法获取系统代理信息</span>
      </div>
    </div>

    <!-- 代理说明 -->

    <div class="proxy-guide" v-show="proxyType=='none'">
      <div class="guide-header" >
        <n-icon size="18" :component="InformationCircleOutline" />
        <span>代理设置说明:</span>
      </div>
      <div class="guide-content">
        <div class="guide-item">
          <div class="item-title"><n-icon size="16" :component="PlanetOutline"></n-icon> 禁用代理（默认）</div>
          <div class="item-desc">无视系统代理设置，始终直连访问。适用于国内用户开启代理无法的情况。</div>
        </div>
        <div class="guide-item">
          <div class="item-title"><n-icon size="16" :component="PlanetOutline"></n-icon> 系统代理</div>
          <div class="item-desc">跟随系统代理设置。适用于不能直接访问国内网站的区域通过操作系统或开启代理软件使用。</div>
        </div>
        <div class="guide-item">
          <div class="item-title"><n-icon size="16" :component="PlanetOutline"></n-icon> 手动代理</div>
          <div class="item-desc">自定义代理服。适用于不能直接访问国内往网站地通过指定代理使用。</div>
        </div>
      </div>
    </div>


    <!-- 测试结果 -->
    <div class="proxy-test-result" v-if="proxyTestResllt">
      <transition name="fade">
        <div :class="['test-result-content', resultType]">
          <div class="status-icon" :class="{ 'loading': testingConnection }">
            <div v-if="testingConnection" class="loading-icon">⟳</div>
            <div v-else-if="resultType === 'info'" class="info-icon">ℹ️</div>
            <div v-else-if="resultType === 'error'" class="error-icon">❌</div>
            <div v-else-if="resultType === 'success'" class="success-icon">✅</div>
          </div>
          <pre>{{ proxyTestResllt }}</pre>
        </div>
      </transition>
    </div>

    <!-- 操作按钮 -->
    <div class="actions-section">
      <hr style="margin: 16px 0; border: none; border-top: 1px solid #eee;" />
      <div class="button-group">
        <n-button 
          size="small" 
          @click="testConnection" 
          :loading="testingConnection"
          type="primary"
        >
          测试连接
        </n-button>
        <!-- <n-button 
          @click="resetSettings" 
          :loading="resetting" 
          :disabled="testingConnection"
          size="small"
          secondary
        >
          重置为不使用代理
        </n-button> -->
      </div>
    </div>
    <div v-if="proxyStore.localProxyPort" class="proxy-port-info" style="margin-top:12px;color:#888;font-size:13px;">
      当前本地代理服务器端口：{{ proxyStore.localProxyPort }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue';
import { 
  NRadioGroup, 
  NRadio, 
  NSpace, 
  NInput, 
  NButton,
  
  NIcon
} from 'naive-ui';
import { useProxyStore } from '../store/useProxyStore';
import { invoke } from '@tauri-apps/api/core';

import type { SystemProxyInfo } from '../types/proxy';
import { InformationCircleOutline,PlanetOutline } from '@vicons/ionicons5';

const proxyStore = useProxyStore();
const loading = ref(false);
const proxyEnabled = ref(false);
const statusMessage = ref('');
const statusType = ref<'success' | 'error' | 'warning'>('warning');

const proxyTestResllt = ref('');
const resultType = ref('info');
const testingConnection = ref(false);
// const resetting = ref(false);
const resultMessage = ref('');

// 系统代理信息相关
const systemProxyInfo = ref({
  http_proxy: '',
  https_proxy: '',
  socks_proxy: '',
  ftp_proxy: '',
  no_proxy: '',
  proxy_enabled: false
});
const loadingSystemProxy = ref(false);

// 校验状态
interface ValidationState {
  status: 'success' | 'warning' | 'error' | undefined;
  message: string;
}

const httpProxyValidation = ref<ValidationState>({ status: undefined, message: '' });
const httpsProxyValidation = ref<ValidationState>({ status: undefined, message: '' });
const socksProxyValidation = ref<ValidationState>({ status: undefined, message: '' });

// 自动清除结果消息
const clearResultAfterDelay = (delay: number) => {
  setTimeout(() => {
    resultMessage.value = '';
    resultType.value = '';
  }, delay);
};

// 防抖函数
function debounce(fn: Function, delay: number) {
  let timeoutId: ReturnType<typeof setTimeout>;
  return function(this: any, ...args: any[]) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn.apply(this, args), delay);
  };
}

// 校验函数
const debouncedValidateHttpProxy = debounce(async (value: string) => {
  if (!value) {
    httpProxyValidation.value = { status: undefined, message: '' };
    return;
  }
  const result = proxyStore.validateProxyAddress(value);
  httpProxyValidation.value = {
    status: result.valid ? 'success' : 'error',
    message: result.valid ? '格式正确' : result.message
  };
}, 300);

const debouncedValidateHttpsProxy = debounce(async (value: string) => {
  if (!value) {
    httpsProxyValidation.value = { status: undefined, message: '' };
    return;
  }
  const result = proxyStore.validateProxyAddress(value);
  httpsProxyValidation.value = {
    status: result.valid ? 'success' : 'error',
    message: result.valid ? '格式正确' : result.message
  };
}, 300);

const debouncedValidateSocksProxy = debounce(async (value: string) => {
  if (!value) {
    socksProxyValidation.value = { status: undefined, message: '' };
    return;
  }
  const result = proxyStore.validateProxyAddress(value);
  socksProxyValidation.value = {
    status: result.valid ? 'success' : 'error',
    message: result.valid ? '格式正确' : result.message
  };
}, 300);

const proxyType = computed({
  get: () => proxyStore.config.type,
  set: async (value) => {
    try {
      await proxyStore.setType(value);
      // 如果是系统代理，自动应用系统代理设置
      if (value === 'system') {
        await refreshSystemProxy();
        await applySystemProxySettings();
      }
    } catch (e) {
      console.error('[ProxySettings] ❌ 设置代理类型失败:', e);
      resultMessage.value = `设置代理类型失败: ${e}`;
      resultType.value = 'error';
      clearResultAfterDelay(5000);
    }
  }
});

const httpProxy = computed({
  get: () => proxyStore.config.httpProxy,
  set: (value) => {
    proxyStore.setHTTPProxy(value);
  }
});

const httpsProxy = computed({
  get: () => proxyStore.config.httpsProxy,
  set: (value) => {
    proxyStore.setHTTPSProxy(value);
  }
});

const socksProxy = computed({
  get: () => proxyStore.config.socksProxy,
  set: (value) => {
    proxyStore.setSOCKSProxy(value);
  }
});

// noProxy 功能已移除，保留模板兼容性
const noProxy = ref('');

// 智能应用到全部
async function applyToAll(sourceAddress: string, _sourceType?: 'http' | 'https' | 'socks') {
  // 确保源地址有值
  if (!sourceAddress) {
    resultMessage.value = '源地址不能为空';
    resultType.value = 'error';
    clearResultAfterDelay(5000);
    return;
  }

  const success = await proxyStore.applyToAll(sourceAddress);
  if (success) {
    resultMessage.value = '已应用到所有代理类型';
    resultType.value = 'success';
    
    // 重新校验所有字段
    debouncedValidateHttpProxy(proxyStore.config.httpProxy);
    debouncedValidateHttpsProxy(proxyStore.config.httpsProxy);
    debouncedValidateSocksProxy(proxyStore.config.socksProxy);
  } else {
    resultMessage.value = proxyStore.error || '应用失败';
    resultType.value = 'error';
  }
  clearResultAfterDelay(5000);
}

// 清除所有代理
function clearAllProxies() {
  proxyStore.clearAll();
  httpProxyValidation.value = { status: undefined, message: '' };
  httpsProxyValidation.value = { status: undefined, message: '' };
  socksProxyValidation.value = { status: undefined, message: '' };
  
  resultMessage.value = '已清除所有代理配置';
  resultType.value = 'info';
  clearResultAfterDelay(5000);
}

// 获取系统代理信息
const refreshSystemProxy = async () => {
  try {
    loadingSystemProxy.value = true;
    const proxyInfo = await invoke('get_system_proxy_info');
    systemProxyInfo.value = proxyInfo as SystemProxyInfo;
    
    console.log('[ProxySettings] 系统代理信息:', proxyInfo);
  } catch (error) {
    console.error('获取系统代理信息失败:', error);
  } finally {
    loadingSystemProxy.value = false;
  }
};

 
// 新增：应用系统代理设置
const applySystemProxySettings = async () => {
  try {
    loadingSystemProxy.value = true;
    resultMessage.value = '';
    
    const result = await proxyStore.applySystemProxy();
    if (result.success) {
      resultMessage.value = '系统代理设置已应用';
      resultType.value = 'success';
      
      // 刷新系统代理信息显示
      await refreshSystemProxy();
    } else {
      resultMessage.value = result.message;
      resultType.value = 'error';
    }
  } catch (error) {
    resultMessage.value = `应用系统代理失败: ${error}`;
    resultType.value = 'error';
  } finally {
    loadingSystemProxy.value = false;
    clearResultAfterDelay(5000);
  }
};

// 新增：测试单个代理
const testSingleProxy = async (proxyUrl: string, proxyName: string) => {
  if (!proxyUrl) {
    return {
      success: false,
      message: `${proxyName}代理地址为空`
    };
  }
  
  try {
    testingConnection.value = true;
    resultMessage.value = `正在测试${proxyName}代理...`;
    resultType.value = 'info';
    
    return await proxyStore.testProxyConnectivity(proxyUrl);
  } catch (error) {
    return {
      success: false,
      message: `${proxyName}代理测试异常: ${error}`
    };
  }
};

// 重置测试结果
function resetTestResult() {
  proxyTestResllt.value = '';
  resultType.value = 'info';
  testingConnection.value = false;
}

// 监听代理类型变化
watch(proxyType, () => {
  resetTestResult();
});

// 测试连接（改进版）
async function testConnection() {
  resetTestResult();
  testingConnection.value = true;
  proxyTestResllt.value = '正在测试连接...';
  resultType.value = 'info';

  try {
    // 根据当前代理类型进行不同的测试
    if (proxyType.value === 'none') {
      // 在禁用代理模式下直接测试网站可访问性
      const testResult = await proxyStore.testProxyConnectivity('direct://');
      if (testResult.details) {
        proxyTestResllt.value = `直连测试结果：
- core333.com: ${testResult.details.core333_accessible ? '✅ 可访问' : '❌ 不可访问'}
- google.com: ${testResult.details.google_accessible ? '✅ 可访问' : '❌ 不可访问'}`;
        
        resultType.value = testResult.details.core333_accessible || testResult.details.google_accessible ? 'success' : 'error';
      }
    } else if (proxyType.value === 'system') {
      // 测试系统代理
      if (systemProxyInfo.value.proxy_enabled) {
        const proxies = [
          { url: systemProxyInfo.value.http_proxy, name: 'HTTP' },
          { url: systemProxyInfo.value.https_proxy, name: 'HTTPS' },
          { url: systemProxyInfo.value.socks_proxy, name: 'SOCKS' }
        ].filter(p => p.url);
        
        if (proxies.length > 0) {
          const proxy = proxies[0]; // 测试第一个可用的代理
          proxyTestResllt.value = `正在测试系统${proxy.name}代理...`;
          const testResult = await testSingleProxy(proxy.url, `系统${proxy.name}`);
          if (testResult.details) {
            proxyTestResllt.value = `代理测试结果：
- 代理服务器: ${testResult.details.proxy_available ? '✅ 可用' : '❌ 不可用'}
- core333.com: ${testResult.details.core333_accessible ? '✅ 可访问' : '❌ 不可访问'}
- google.com: ${testResult.details.google_accessible ? '✅ 可访问' : '❌ 不可访问'}`;
            
            // 根据测试结果设置状态
            if (testResult.details.proxy_available && 
               (testResult.details.core333_accessible || testResult.details.google_accessible)) {
              resultType.value = 'success';
            } else {
              resultType.value = 'error';
            }
          }
        } else {
          proxyTestResllt.value = '系统代理已启用但未配置具体地址';
          resultType.value = 'error';
        }
      } else {
        proxyTestResllt.value = '系统代理未启用';
        resultType.value = 'error';
      }
    } else {
      // 测试手动代理
      const proxy = getActiveProxy();
      if (proxy) {
        proxyTestResllt.value = `正在测试${proxy.name}代理...`;
        const testResult = await testSingleProxy(proxy.url, proxy.name);
        if (testResult.details) {
          proxyTestResllt.value = `代理测试结果：
- 代理服务器: ${testResult.details.proxy_available ? '✅ 可用' : '❌ 不可用'}
- core333.com: ${testResult.details.core333_accessible ? '✅ 可访问' : '❌ 不可访问'}
- google.com: ${testResult.details.google_accessible ? '✅ 可访问' : '❌ 不可访问'}`;
          
          // 根据测试结果设置状态
          if (testResult.details.proxy_available && 
             (testResult.details.core333_accessible || testResult.details.google_accessible)) {
            resultType.value = 'success';
          } else {
            resultType.value = 'error';
          }
        }
      } else {
        proxyTestResllt.value = '未配置任何手动代理';
        resultType.value = 'error';
      }
    }
  } catch (error) {
    proxyTestResllt.value = `测试失败: ${error}`;
    resultType.value = 'error';
  } finally {
    testingConnection.value = false;
  }
}

// 重置设置
// async function resetSettings() {
//   resetting.value = true;
//   await proxyStore.resetConfig();
  
//   // Clear validation states
//   httpProxyValidation.value = { status: undefined, message: '' };
//   httpsProxyValidation.value = { status: undefined, message: '' };
//   socksProxyValidation.value = { status: undefined, message: '' };

//   resetting.value = false;
// }

// 每次弹窗显示时都初始化
const popoverVisible = ref(false);
watch(popoverVisible, (val) => {
  if (val) proxyStore.initialize();
});

onMounted(() => {
  proxyStore.initialize();
  refreshSystemProxy();
});

watch(proxyType, () => {
  if (proxyType.value === 'system') {
    refreshSystemProxy();
  }
});

// 获取当前活动的代理配置
function getActiveProxy() {
  const proxies = [
    { url: proxyStore.config.httpProxy, name: 'HTTP' },
    { url: proxyStore.config.httpsProxy, name: 'HTTPS' },
    { url: proxyStore.config.socksProxy, name: 'SOCKS5' }
  ].filter(p => p.url);
  
  return proxies.length > 0 ? proxies[0] : null;
}

// 更新状态显示
function updateStatus(message: string, type: 'success' | 'error' | 'warning') {
  statusMessage.value = message;
  statusType.value = type;
}

// 切换代理状态
async function toggleProxy() {
  if (loading.value) return;
  
  try {
    loading.value = true;
    
    if (proxyEnabled.value) {
      // 禁用代理
      await proxyStore.setType('none');
      proxyEnabled.value = false;
      updateStatus('未启用', 'warning');
    } else {
      // 启用代理
      const result = await proxyStore.applyManualProxy();
      if (result.success) {
        proxyEnabled.value = true;
        updateStatus('已启用', 'success');
      } else {
        updateStatus('启用失败', 'error');
      }
    }
  } catch (e) {
    console.error('代理操作失败:', e);
    updateStatus('操作失败', 'error');
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.proxy-settings-popover {
  width: 380px;
  padding: 16px;
  background-color: #ffffff;
  /* border: 1px solid #e0e0e0; */
  /* border-radius: 8px; */
  /* box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15); */
}

.settings-header {
  margin-bottom: 16px;
}

.settings-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #333;
}

.proxy-type-section {
  margin-bottom: 16px;
}

.radio-label {
  font-size: 14px;
  color: #333;
}

.manual-proxy-section {
  margin-bottom: 16px;
}

.form-item {
  margin-bottom: 12px;
}

.form-item label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.input-with-button {
  display: flex;
  gap: 8px;
  align-items: center;
}

.input-with-button .n-input {
  flex: 1;
}

.validation-message {
  margin-top: 4px;
  font-size: 12px;
  line-height: 1.4;
}

.validation-message.success {
  color: #4caf50;
}

.validation-message.error {
  color: #f44336;
}

.validation-message.warning {
  color: #ff9800;
}

.proxy-hint {
  margin-top: 4px;
}

.proxy-hint small {
  color: #666;
  font-size: 12px;
}

.quick-actions {
  margin-top: 12px;
  text-align: right;
}

.system-proxy-section {
  margin-bottom: 16px;
}

.system-proxy-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.system-proxy-actions {
  display: flex;
  gap: 6px;
}

.system-proxy-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.system-proxy-info {
  margin-bottom: 8px;
}

.proxy-info-item {
  margin-bottom: 4px;
}

.proxy-info-item .proxy-label {
  font-size: 12px;
  font-weight: 500;
  color: #333;
}

.proxy-info-item .proxy-value {
  font-size: 12px;
  color: #666;
}

.proxy-info-item.no-proxy .proxy-value {
  color: #d32f2f;
}

.proxy-status-item {
  margin-top: 4px;
}

.proxy-status {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.proxy-status.enabled {
  background: #e8f5e8;
  color: #2e7d32;
}

.loading-info {
  text-align: center;
  color: #666;
}

.no-proxy-info {
  text-align: center;
  color: #d32f2f;
}

.active-proxy-section {
  margin-top: 8px;
}

.active-proxy-header h5 {
  margin: 0 0 6px 0;
  font-size: 13px;
  font-weight: 600;
  color: #1976d2;
}

.proxy-value.active {
  color: #1976d2;
  font-weight: 500;
}

.proxy-note {
  margin-top: 6px;
  padding: 4px 8px;
  background: #fff3cd;
  border: 1px solid #ffeaa7;
  border-radius: 4px;
}

.proxy-note small {
  color: #856404;
  font-size: 11px;
}

.status-section {
  margin-bottom: 16px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.status-tag {
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.status-tag.success {
  background: #e8f5e8;
  color: #2e7d32;
}

.status-tag.loading {
  background: #e3f2fd;
  color: #1976d2;
}

.status-tag.error {
  background: #ffebee;
  color: #d32f2f;
}

.error-message {
  margin-top: 8px;
}

.actions-section {
  margin-bottom: 0;
}

.button-group {
  display: flex;
  gap: 8px;
}

.result-section {
  margin: 12px 0;
}

.result-message {
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 13px;
  line-height: 1.4;
  border: 1px solid;
}

.result-message.success {
  background-color: #f0f9ff;
  color: #0369a1;
  border-color: #bae6fd;
}

.result-message.error {
  background-color: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}

.result-message.info {
  background-color: #f0fdf4;
  color: #16a34a;
  border-color: #bbf7d0;
}

/* 添加样式以正确显示多行文本 */
:deep(.el-message-box__message) {
  white-space: pre-line;
  font-family: monospace;
  margin-top: 8px;
}

.proxy-test-result {
  min-height: 80px;
  margin: 16px -16px -16px;
  font-family: "Fira Code", "Source Code Pro", Consolas, monospace;
}

.test-result-content {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  padding: 16px;
  border-bottom-left-radius: 8px;
  border-bottom-right-radius: 8px;
  transition: all 0.3s ease;
}

.test-result-content pre {
  margin: 0;
  white-space: pre-wrap;
  word-wrap: break-word;
  flex: 1;
  font-size: 14px;
  line-height: 1.6;
}

.status-icon {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  flex-shrink: 0;
}

.info-icon, .error-icon, .success-icon {
  font-size: 16px;
}

.info {
  background-color: rgba(24, 144, 255, 0.1);
  border-top: 1px solid rgba(24, 144, 255, 0.2);
}

.info .status-icon {
  background: rgba(24, 144, 255, 0.15);
}

.error {
  background-color: rgba(255, 77, 79, 0.1);
  border-top: 1px solid rgba(255, 77, 79, 0.2);
}

.error .status-icon {
  background: rgba(255, 77, 79, 0.15);
}

.success {
  background-color: rgba(82, 196, 26, 0.1);
  border-top: 1px solid rgba(82, 196, 26, 0.2);
}

.success .status-icon {
  background: rgba(82, 196, 26, 0.15);
}

/* 淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: all 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-5px);
}

/* 确保图标垂直居中 */
.status-icon > div {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.loading-icon {
  animation: spin 1s linear infinite;
  font-size: 18px;
}

.status-icon.loading {
  background: rgba(24, 144, 255, 0.15);
}

/* 测试中状态样式 */
.info.testing {
  background-color: rgba(24, 144, 255, 0.05);
}

.info.testing .status-icon {
  background: rgba(24, 144, 255, 0.1);
}

.proxy-guide {
  background: #f8fafc;
  border-radius: 8px;
  padding: 16px;
  margin: 16px 0;
}

.guide-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  color: var(--n-text-color-2);
  font-weight: bold;
}

.guide-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.guide-item {
  padding: 8px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.guide-item:hover {
  background-color: var(--n-table-color-hover);
}

.item-title {
  font-weight: bold;
  margin-bottom: 4px;
  color: var(--n-text-color-1);
}

.item-desc {
  font-size: 13px;
  color: var(--n-text-color-3);
  line-height: 1.5;
}
</style> 