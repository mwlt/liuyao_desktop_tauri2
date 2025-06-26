<template>
  <div class="settings-panel">
    <h2>代理设置</h2>
    
    <!-- 测试基本渲染 -->
    <div class="test-section">
      <p>设置页面加载成功！</p>
      <p>代理状态: {{ proxyStore.config.type }}</p>
      <p>代理地址: {{ proxyStore.config.address || '无' }}</p>
    </div>

    <!-- 基本的代理配置 -->
    <div class="proxy-config">
      <h3>代理类型</h3>
      <div class="radio-group">
        <label>
          <input 
            type="radio" 
            value="none" 
            v-model="proxyType"
          />
          禁用代理
        </label>
        <label>
          <input 
            type="radio" 
            value="system" 
            v-model="proxyType"
          />
          系统代理
        </label>
        <label>
          <input 
            type="radio" 
            value="manual" 
            v-model="proxyType"
          />
          手动代理
        </label>
      </div>

      <!-- 手动代理输入 -->
      <div v-if="proxyType === 'manual'" class="manual-proxy">
        <h3>代理地址</h3>
        <input 
          type="text"
          v-model="proxyAddress"
          placeholder="如: http://127.0.0.1:8888"
          class="proxy-input"
        />
      </div>

      <!-- 状态显示 -->
      <div class="status-section">
        <h3>当前状态</h3>
        <p v-if="proxyStore.isLoading">配置中...</p>
        <p v-else-if="proxyStore.error" class="error">{{ proxyStore.error }}</p>
        <p v-else class="success">配置正常</p>
      </div>

      <!-- 测试按钮 -->
      <div class="actions">
        <button @click="testConnection" :disabled="proxyStore.isLoading">
          测试连接
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted } from 'vue';
import { useProxyStore } from '../store/useProxyStore';

const proxyStore = useProxyStore();

const proxyType = computed({
  get: () => proxyStore.config.type,
  set: (val) => proxyStore.setType(val as any)
});

const proxyAddress = computed({
  get: () => proxyStore.config.address,
  set: (val) => proxyStore.setHTTPProxy(val)
});

// 测试连接
const testConnection = async () => {
  console.log('测试网络连接...');
  if (window.go && window.go.main && window.go.main.App && window.go.main.App.GetNetworkStatus) {
    try {
      const status = await window.go.main.App.GetNetworkStatus();
      console.log('网络状态:', status);
      alert(`连接${status.canReachTarget ? '成功' : '失败'}: ${status.errorMessage || '正常'}`);
    } catch (error) {
      console.error('测试失败:', error);
      alert(`测试失败: ${error}`);
    }
  }
};

// 生命周期
onMounted(async () => {
  console.log('Settings组件挂载');
  // 初始化代理配置
  await proxyStore.initFromBackend();
  // 监听后端事件
  proxyStore.listenToBackendEvents();
});

onUnmounted(() => {
  console.log('Settings组件卸载');
  proxyStore.cleanup();
});
</script>

<style scoped>
.settings-panel {
  max-width: 600px;
  margin: 40px auto;
  padding: 24px;
}

.test-section {
  background: #f0f8ff;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 24px;
  border: 1px solid #e0e0e0;
}

.proxy-config {
  background: #fff;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.radio-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin: 16px 0;
}

.radio-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.manual-proxy {
  margin: 16px 0;
}

.proxy-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.status-section {
  margin: 16px 0;
  padding: 12px;
  background: #f9f9f9;
  border-radius: 4px;
}

.error {
  color: #d32f2f;
}

.success {
  color: #2e7d32;
}

.actions {
  margin-top: 16px;
}

.actions button {
  background: #1976d2;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.actions button:hover:not(:disabled) {
  background: #1565c0;
}

.actions button:disabled {
  background: #ccc;
  cursor: not-allowed;
}

h2, h3 {
  color: #333;
  margin-bottom: 16px;
}
</style>
