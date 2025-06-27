<template>
  <div class="main-content">
    <!-- 主内容区域 -->
    <div class="content-area">
      <template v-if="networkStatus && networkStatus.isOnline && networkStatus.canReachTarget">
        <iframe
          src="http://www.core333.com"
          frameborder="0"
          class="content-iframe"
          @load="onFrameLoad"
          @error="onFrameError"
        ></iframe>
      </template>
      <template v-else>
        <div class="error-display">
          <div class="error-title">⚠️ 网络异常</div>
          <div class="error-message">{{ errorMessage }}</div>
          <div class="error-actions">
            <button @click="refresh">刷新</button>
          </div>
          <div class="error-detail">
            <div>最后检查: {{ networkStatus ? networkStatus.lastCheck : '' }}</div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
// import { invoke } from '@tauri-apps/api/core';

// const props = defineProps<{ networkStatus: any }>();

interface NetworkStatus {
  isOnline: boolean;
  canReachTarget: boolean;
  errorMessage: string;
  lastCheck: string;
  localProxyUrl: string;
  isLoading: boolean;
  isError: boolean;
  isSuccess: boolean;
  isOffline: boolean;
}
const networkStatus = ref<NetworkStatus>({
  isOnline: true,
  canReachTarget: true,
  errorMessage: '无法访问目标网站',
  lastCheck: '2025-06-25 19:21:00',
  localProxyUrl: 'http://127.0.0.1:8080',
  isLoading: false,
  isError: false,
  isSuccess: false,
  isOffline: false,
 
});

// 在组件挂载时设置本地代理服务器地址
onMounted(async () => {
  
  // 使用默认的本地代理地址
  networkStatus.value.localProxyUrl = 'http://www.core333.com';
});

const errorMessage = computed(() => {
  if (!networkStatus.value) return '未检测到网络状态';
  if (!networkStatus.value.isOnline) return '无法连接互联网';
  if (!networkStatus.value.canReachTarget) return networkStatus.value.errorMessage || '无法访问目标网站';
  return '';
});



function onFrameLoad() {
  console.log('页面加载完成');
}

function onFrameError() {
  console.error('页面加载失败');
}

function refresh() {
  // 刷新iframe
  const iframe = document.querySelector('.content-iframe') as HTMLIFrameElement;
  if (iframe) {
    iframe.src = iframe.src;
  }
}
</script>

<style scoped>
.main-content {
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}



.content-area {
  flex-grow: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.content-iframe {
  width: 100%;
  height: 100%;
  border: none;
}

.error-display {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  color: #b71c1c;
  background: #fff5f5;
  border: 1px solid #ffcdd2;
  border-radius: 8px;
  padding: 32px 16px;
  margin: 16px;
}

.error-title {
  font-size: 1.5rem;
  font-weight: bold;
  margin-bottom: 12px;
}

.error-message {
  font-size: 1.1rem;
  margin-bottom: 16px;
  text-align: center;
}

.error-actions {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.error-actions button {
  background: #f44336;
  color: #fff;
  border: none;
  border-radius: 4px;
  padding: 8px 20px;
  font-size: 1rem;
  cursor: pointer;
  transition: background 0.2s;
}

.error-actions button:hover {
  background: #b71c1c;
}

.error-detail {
  font-size: 0.9rem;
  color: #888;
  text-align: center;
  line-height: 1.4;
}
</style> 