<template>
  <div class="proxy-settings-popover">
    <div v-if="resultMessage" :class="['result-message', `result-message--${resultType}`]">
      {{ resultMessage }}
    </div>
    <div  class="proxy-port-info" style="margin-top:12px;color:#888;font-size:13px;">
      当前本地代理端口：{{ proxyStore.localProxyPort }}
    </div>
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
        <n-button size="tiny" @click="refreshSystemProxy" :loading="loadingSystemProxy" quaternary>
          刷新
        </n-button>
      </div>
      
      <div v-if="systemProxyInfo" class="system-proxy-info">
        <div class="proxy-info-item">
          <span class="proxy-label">HTTP代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.httpProxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">HTTPS代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.httpsProxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">SOCKS代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.socksProxy }}</span>
        </div>
        <div class="proxy-info-item">
          <span class="proxy-label">FTP代理:</span>
          <span class="proxy-value">{{ systemProxyInfo.ftpProxy }}</span>
        </div>
        <div v-if="systemProxyInfo.noProxy" class="proxy-info-item">
          <span class="proxy-label">代理例外:</span>
          <span class="proxy-value no-proxy">{{ systemProxyInfo.noProxy }}</span>
        </div>
        <div class="proxy-status-item">
          <span class="proxy-label">状态:</span>
          <span class="proxy-status" :class="{ enabled: systemProxyInfo.proxyEnabled }">
            {{ systemProxyInfo.proxyEnabled ? '已启用' : '未启用' }}
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
        <n-button 
          @click="resetSettings" 
          :loading="resetting" 
          :disabled="testingConnection"
          size="small"
          secondary
        >
          重置为不使用代理
        </n-button>
      </div>
    </div>
    <div v-if="proxyStore.localProxyPort" class="proxy-port-info" style="margin-top:12px;color:#888;font-size:13px;">
      当前本地代理端口：{{ proxyStore.localProxyPort }}
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
 
} from 'naive-ui';
import { useProxyStore } from '../store/useProxyStore';
import { invoke } from '@tauri-apps/api/core';

const proxyStore = useProxyStore();

const testingConnection = ref(false);
const resetting = ref(false);
const resultMessage = ref('');
const resultType = ref(''); // 'success', 'error', 'info'

// 系统代理信息相关
interface SystemProxyInfo {
  httpProxy: string;
  httpsProxy: string;
  socksProxy: string;
  ftpProxy: string;
  noProxy: string;
  proxyEnabled: boolean;
}
const systemProxyInfo = ref<SystemProxyInfo>({
  httpProxy: '',
  httpsProxy: '',
  socksProxy: '',
  ftpProxy: '',
  noProxy: '',
  proxyEnabled: false
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
const clearResultAfterDelay = () => {
  setTimeout(() => {
    resultMessage.value = '';
    resultType.value = '';
  }, 5000);
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
  const result = await proxyStore.validateProxy(value);
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
  const result = await proxyStore.validateProxy(value);
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
  const result = await proxyStore.validateProxy(value);
  socksProxyValidation.value = {
    status: result.valid ? 'success' : 'error',
    message: result.valid ? '格式正确' : result.message
  };
}, 300);

const proxyType = computed({
  get: () => proxyStore.config.type,
  set: (value) => proxyStore.setType(value as any)
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

const noProxy = computed({
  get: () => proxyStore.config.noProxy,
  set: (value) => proxyStore.setNoProxy(value)
});

// 智能应用到全部
async function applyToAll(sourceAddress: string, sourceType: 'http' | 'https' | 'socks') {
  // 确保源地址有值
  if (!sourceAddress) {
    resultMessage.value = '源地址不能为空';
    resultType.value = 'error';
    clearResultAfterDelay();
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
  clearResultAfterDelay();
}

// 清除所有代理
function clearAllProxies() {
  proxyStore.clearAll();
  httpProxyValidation.value = { status: undefined, message: '' };
  httpsProxyValidation.value = { status: undefined, message: '' };
  socksProxyValidation.value = { status: undefined, message: '' };
  
  resultMessage.value = '已清除所有代理配置';
  resultType.value = 'info';
  clearResultAfterDelay();
}

// 获取系统代理信息
const refreshSystemProxy = async () => {
  try {
    loadingSystemProxy.value = true;
    const info = await invoke('get_system_proxy_info');
    systemProxyInfo.value = info  as SystemProxyInfo;
  } catch (error) {
    console.error('Failed to get system proxy info:', error);
  } finally {
    loadingSystemProxy.value = false;
  }
};

// 监听代理类型变化
watch(proxyType, (newType) => {
  if (newType === 'system') {
    refreshSystemProxy();
  }
});

// 测试连接
async function testConnection() {
  testingConnection.value = true;
  try {
    if (window.go?.main?.App?.GetNetworkStatus) {
      const status = await window.go.main.App.GetNetworkStatus();
      if (status.canReachTarget) {
        resultMessage.value = `连接测试成功 (${status.responseTime}ms)`;
        resultType.value = 'success';
      } else {
        resultMessage.value = `连接测试失败: ${status.errorMessage}`;
        resultType.value = 'error';
      }
    }
  } catch (error) {
    resultMessage.value = `测试失败: ${error}`;
    resultType.value = 'error';
  } finally {
    testingConnection.value = false;
    clearResultAfterDelay();
  }
}

// 重置设置
async function resetSettings() {
  resetting.value = true;
  await proxyStore.resetConfig();
  
  // Clear validation states
  httpProxyValidation.value = { status: undefined, message: '' };
  httpsProxyValidation.value = { status: undefined, message: '' };
  socksProxyValidation.value = { status: undefined, message: '' };

  resetting.value = false;
}

// 每次弹窗显示时都拉取端口
const popoverVisible = ref(false);
watch(popoverVisible, (val) => {
  if (val) proxyStore.initLocalProxyPort();
});

onMounted(() => {
  proxyStore.initLocalProxyPort();
});

watch(proxyType, () => {
  if (proxyType.value === 'system') {
    refreshSystemProxy();
  }
});
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
</style> 