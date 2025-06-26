<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-25 19:20:01
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-27 00:31:59
 * @FilePath: \liuyao_desktop_tauri\src\components\NavBar.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<template>
  <div class="navbar">
    <!-- 左侧按钮组 -->
    <div class="left-actions">
      <n-button 
        size="small"
        @click="openInDefaultBrowser"
        quaternary
        class="nav-button"
      >
        <template #icon>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14,3V5H17.59L7.76,14.83L9.17,16.24L19,6.41V10H21V3M19,19H5V5H12V3H5C3.89,3 3,3.9 3,5V19A2,2 0 0,0 5,21H19A2,2 0 0,0 21,19V12H19V19Z" />
          </svg>
        </template>
        在默认浏览器打开
      </n-button>
      
      <n-button 
        size="small"
        @click="refreshPage"
        quaternary
        class="nav-button"
      >
        <template #icon>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor">
            <path d="M17.65,6.35C16.2,4.9 14.21,4 12,4A8,8 0 0,0 4,12A8,8 0 0,0 12,20C15.73,20 18.84,17.45 19.73,14H17.65C16.83,16.33 14.61,18 12,18A6,6 0 0,1 6,12A6,6 0 0,1 12,6C13.66,6 15.14,6.69 16.22,7.78L13,11H20V4L17.65,6.35Z" />
          </svg>
        </template>
        刷新(F5)
      </n-button>
    </div>

    <!-- <span class="status-dot" :class="statusClass" />
    <h1>liuyao_desktop2</h1> -->
    
    <!-- 状态显示区域 -->
    <div class="status-info">
      <div class="network-status">
        <span class="proxy-label">网络状态：</span>
        <span class="status-indicator" :class="networkStatusClass">●</span>
        <span class="status-text"  :class="networkStatusClass">{{ networkStatusText }}</span>
        <span v-if="networkStatus && networkStatus.responseTime" class="response-time">
          ({{ networkStatus.responseTime }}ms)
        </span>
      </div>
      
      <div class="proxy-status">
        <span class="proxy-label">代理:</span>
        <span class="proxy-badge" :class="proxyStatusClass">{{ proxyStatusText }}</span>
      </div>
    </div>
    
    
    <div class="nav-actions">
      <!-- 代理设置弹出框 -->
      <n-popover 
        trigger="hover" 
        placement="bottom-end"
        :show-arrow="true"
        :z-index="1000"
      >
        <template #trigger>
          <n-button 
            size="small" 
            text
            class="nav-button"
            :class="{ 'proxy-active': isProxyActive }"
          >
            <template #icon>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12,15.5A3.5,3.5 0 0,1 8.5,12A3.5,3.5 0 0,1 12,8.5A3.5,3.5 0 0,1 15.5,12A3.5,3.5 0 0,1 12,15.5M19.43,12.97C19.47,12.65 19.5,12.33 19.5,12C19.5,11.67 19.47,11.34 19.43,11.03L21.54,9.37C21.73,9.22 21.78,8.95 21.66,8.73L19.66,5.27C19.54,5.05 19.27,4.96 19.05,5.05L16.56,6.05C16.04,5.66 15.5,5.32 14.87,5.07L14.5,2.42C14.46,2.18 14.25,2 14,2H10C9.75,2 9.54,2.18 9.5,2.42L9.13,5.07C8.5,5.32 7.96,5.66 7.44,6.05L4.95,5.05C4.73,4.96 4.46,5.05 4.34,5.27L2.34,8.73C2.22,8.95 2.27,9.22 2.46,9.37L4.57,11.03C4.53,11.34 4.5,11.67 4.5,12C4.5,12.33 4.53,12.65 4.57,12.97L2.46,14.63C2.27,14.78 2.22,15.05 2.34,15.27L4.34,18.73C4.46,18.95 4.73,19.03 4.95,18.95L7.44,17.94C7.96,18.34 8.5,18.68 9.13,18.93L9.5,21.58C9.54,21.82 9.75,22 10,22H14C14.25,22 14.46,21.82 14.5,21.58L14.87,18.93C15.5,18.68 16.04,18.34 16.56,17.94L19.05,18.95C19.27,19.03 19.54,18.95 19.66,18.73L21.66,15.27C21.78,15.05 21.73,14.78 21.54,14.63L19.43,12.97Z" />
              </svg>
            </template>
            代理设置
          </n-button>
        </template>
       <template #default>
         <ProxySettingsPopover/>
       </template>
       
      </n-popover>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { NPopover, NButton, NCheckbox } from 'naive-ui';
import { useProxyStore } from '../store/useProxyStore';
import ProxySettingsPopover from './ProxySettingsPopover.vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-shell';

// 在系统默认浏览器中打开
const openInDefaultBrowser = async () => {
  try {
    await open('http://www.core333.com');
  } catch (error) {
    console.error('打开浏览器失败:', error);
  }
};

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
  responseTime?: number;
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
  responseTime: 0,
});

const proxyStore = useProxyStore();

// 刷新页面
const refreshPage = () => {
  window.location.reload();
};

// 网络状态相关
const networkStatusClass = computed(() => {
  if (!networkStatus.value) return 'status-unknown';
  if (networkStatus.value.isOnline && networkStatus.value.canReachTarget) return 'status-online';
  if (networkStatus.value.isOnline) return 'status-warning';
  return 'status-offline';
});

const networkStatusText = computed(() => {
  if (!networkStatus.value) return '检测中...';
  if (!networkStatus.value.isOnline) return '离线';
  if (!networkStatus.value.canReachTarget) return '目标不可达';
  return '正常';
});

// 代理状态相关
const proxyStatusClass = computed(() => {
  if (proxyStore.isLoading) return 'proxy-loading';
  if (proxyStore.error) return 'proxy-error';
  return 'proxy-normal';
});

const proxyStatusText = computed(() => {
  if (proxyStore.isLoading) return '配置中...';
  if (proxyStore.error) return '配置错误';
  
  switch (proxyStore.config.type) {
    case 'none': return '禁用';
    case 'system': return '系统代理';
    case 'manual': return `手动 (${proxyStore.config.address})`;
    default: return '未知';
  }
});

// 检查代理是否激活（非禁用状态）
const isProxyActive = computed(() => {
  return proxyStore.config.type !== 'none';
});

</script>

<style scoped>
.navbar {
  height: 50px;
  background: #333333; /* 经典深灰色 - 与 titlebar 匹配 */
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  display: flex;
  align-items: center;
  padding: 0 20px;
  flex-shrink: 0; /* Prevents the navbar from shrinking */
  color: white;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15); /* 柔和的深色阴影 */
}

/* 左侧按钮组样式 */
.left-actions {
  display: flex;
  gap: 8px;
  margin-right: 16px;
}

.nav-button {
  display: flex !important;
  align-items: center !important;
  gap: 4px !important;
  padding: 4px 8px !important;
  border-radius: 4px !important;
  font-size: 12px !important;
  color: rgba(255, 255, 255, 0.9) !important;
  transition: all 0.2s !important;
  white-space: nowrap !important;
}

.nav-button:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: white !important;
}

.navbar h1 {
  margin: 0 0 0 12px;
  font-size: 1.2rem;
  font-weight: 600;
  color: white;
}

.status-dot {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  display: inline-block;
  margin-right: 8px;
  border: 1.5px solid #bbb;
  background: #ccc;
}
.dot-online {
  background: #4caf50;
  border-color: #388e3c;
}
.dot-offline {
  background: #f44336;
  border-color: #b71c1c;
}
.dot-unknown {
  background: #bbb;
  border-color: #888;
}
.nav-actions {
  display: flex;
  gap: 10px;
 
}
.nav-actions button {
  background: none;
  border: none;
  font-size: 1rem;
  padding: 6px 16px;
  border-radius: 4px;
  cursor: pointer;
  color: #333;
  transition: background 0.2s;
}
.nav-actions button.active, .nav-actions button:hover {
  background: #e0e0e0;
}

.settings-button {
  display: flex !important;
  align-items: center !important;
  gap: 6px !important;
  padding: 6px 12px !important;
  border-radius: 4px !important;
  font-size: 14px !important;
  color: rgba(255, 255, 255, 0.9) !important;
  transition: all 0.2s !important;
}

.settings-button:hover {
  background: rgba(255, 255, 255, 0.15) !important;
  color: white !important;
}

.settings-button.proxy-active {
  color: #fff !important;
  background: rgba(255, 255, 255, 0.25) !important;
}

.settings-button.proxy-active:hover {
  background: rgba(255, 255, 255, 0.35) !important;
}

/* 状态信息样式 */
.status-info {
  display: flex;
  align-items: center;
  gap: 20px;
  margin-left: auto;
  margin-right: 20px;
  font-size: 0.85rem;
  color: rgba(255, 255, 255, 0.9);
}

.network-status, .proxy-status {
  display: flex;
  align-items: center;
  gap: 6px;
}

.status-indicator {
  font-size: 1rem;
}

.status-online { color: #4caf50; }
.status-warning { color: #ff9800; }
.status-offline { color: #f44336; }
.status-unknown { color: #9e9e9e; }

.status-text {
  font-weight: 500;
}

.response-time {
  color: #999;
  font-size: 0.75rem;
}

.proxy-label {
  font-weight: 500;
  color: rgba(255, 255, 255, 0.9);
}

.proxy-badge {
  padding: 2px 6px;
  border-radius: 8px;
  font-size: 0.75rem;
  font-weight: 500;
}

.proxy-normal {
  /* background: rgba(76, 175, 80, 0.8); */
  /* color: white; */
  color: rgba(76, 175, 80, 0.8);
  

}

.proxy-loading {
  /* background: rgba(33, 150, 243, 0.8); */
  /* color: white; */
  color: rgba(33, 150, 243, 0.8);
}

.proxy-error {
  /* background: rgba(244, 67, 54, 0.8); */
  /* color: white; */
  color: rgba(244, 67, 54, 0.8);
}
</style> 