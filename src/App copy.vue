<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-23 00:19:54
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-23 21:16:27
 * @FilePath: \liuyao_desktop2\frontend\src\App.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import NavBar from './components/NavBar.vue';
import MainContent from './components/MainContent.vue';
import { useProxyStore } from './store/useProxyStore';
import { GetNetworkStatus } from './wailsjs/go/main/App';
import { EventsOn, EventsOff } from './wailsjs/runtime/runtime';
import type { main } from './wailsjs/go/main/models';
import { initStaticConfig } from './config/staticConfig'

// Reactive data
const networkStatus = ref<main.NetworkStatus | null>(null)

// 初始化代理store
const proxyStore = useProxyStore()

function handleNetworkStatus(status: main.NetworkStatus) {
  networkStatus.value = status
  console.log('网络状态更新:', status)
}

// Lifecycle
onMounted(async () => {
  console.log('应用启动')
  
  try {
    // 1. 初始化静态资源配置（最优先）
    await initStaticConfig()
    console.log('静态资源配置初始化完成')
    
    // 2. 监听网络状态
    EventsOn('network-status', handleNetworkStatus)
    console.log('网络状态监听已启动')
    
    // 3. 获取初始网络状态
    const status = await GetNetworkStatus()
    handleNetworkStatus(status)
    console.log('网络状态获取完成')
    
    // 4. 初始化代理配置
    console.log('初始化代理配置...')
    await proxyStore.initFromBackend()
    proxyStore.listenToBackendEvents()
    console.log('代理配置初始化完成')
    
    console.log('应用初始化完成')
  } catch (error) {
    console.error('初始化失败:', error)
    // 即使初始化失败，也要保持网络状态监听
    EventsOn('network-status', handleNetworkStatus)
  }
});

onUnmounted(() => {
  // Clean up event listeners
  EventsOff("network-status")
  
  // 清理代理store
  proxyStore.cleanup()
  
  console.log('应用清理完成')
})
</script>

<template>
  <div id="app-container">
    <NavBar :network-status="networkStatus" />
    <MainContent :network-status="networkStatus" />
  </div>
</template>

<style>
#app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}
</style>

<style scoped>
.app-container {
  position: relative;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.content-frame {
  width: 100%;
  height: calc(100vh - 50px);
  border: none;
  overflow: auto;
}

.debug-panel-floating {
  position: fixed;
  top: 20px;
  right: 20px;
  width: 320px;
  max-height: 80vh;
  background: rgba(245, 245, 245, 0.95);
  backdrop-filter: blur(10px);
  border: 1px solid #ddd;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
  z-index: 9999;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.debug-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: rgba(51, 51, 51, 0.9);
  color: white;
  border-radius: 12px 12px 0 0;
}

.debug-header h3 {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
}

.close-btn {
  background: none;
  border: none;
  color: white;
  font-size: 18px;
  cursor: pointer;
  padding: 0;
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.debug-content {
  max-height: calc(80vh - 60px);
  overflow-y: auto;
  padding: 12px;
}

.debug-section {
  margin-bottom: 12px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 8px;
  padding: 12px;
  border: 1px solid rgba(0, 0, 0, 0.1);
}

.debug-section h4 {
  margin: 0 0 8px 0;
  color: #333;
  font-size: 12px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-indicator {
  padding: 8px 12px;
  margin-bottom: 8px;
}

.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: #dc3545;
  transition: background-color 0.3s ease;
}

.status-dot.online {
  background: #28a745;
  box-shadow: 0 0 8px rgba(40, 167, 69, 0.5);
}

.status-text {
  font-weight: 600;
  font-size: 12px;
  flex: 1;
}

.check-count {
  font-family: monospace;
  font-size: 11px;
  color: #666;
  background: rgba(0, 0, 0, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
}

.status-grid {
  display: grid;
  gap: 4px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 2px 0;
  font-size: 11px;
}

.label {
  font-weight: 500;
  color: #666;
}

.status.online, .status.reachable {
  color: #28a745;
}

.status.offline, .status.unreachable {
  color: #dc3545;
}

.value {
  font-family: monospace;
  background: rgba(0, 0, 0, 0.1);
  padding: 1px 4px;
  border-radius: 2px;
  font-size: 10px;
}

.error-message {
  margin-top: 8px;
  padding: 6px;
  background: #f8d7da;
  color: #721c24;
  border-radius: 4px;
  font-size: 10px;
}

.button-group {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.test-btn {
  background: #007bff;
  color: white;
  border: none;
  padding: 6px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 10px;
  flex: 1;
  min-width: 60px;
}

.test-btn:hover {
  background: #0056b3;
}

.proxy-btn {
  background: #6c757d;
}

.proxy-btn.enabled {
  background: #28a745;
}

.proxy-btn:hover {
  background: #5a6268;
}

.proxy-btn.enabled:hover {
  background: #218838;
}

.log-container-compact {
  max-height: 100px;
  overflow-y: auto;
  background: rgba(248, 249, 250, 0.8);
  border-radius: 4px;
  padding: 6px;
  font-family: monospace;
  font-size: 9px;
}

.log-item-compact {
  display: flex;
  gap: 6px;
  padding: 1px 0;
  border-bottom: 1px solid rgba(233, 236, 239, 0.5);
}

.log-item-compact:last-child {
  border-bottom: none;
}

.log-time-compact {
  color: #6c757d;
  white-space: nowrap;
  min-width: 60px;
}

.log-message-compact {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.log-item-compact.success .log-message-compact {
  color: #28a745;
}

.log-item-compact.error .log-message-compact {
  color: #dc3545;
}

.log-item-compact.info .log-message-compact {
  color: #333;
}

.clear-btn-small {
  background: #6c757d;
  color: white;
  border: none;
  padding: 2px 6px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 9px;
}

.debug-toggle-floating {
  position: fixed;
  top: 20px;
  right: 20px;
  background: rgba(0, 123, 255, 0.9);
  color: white;
  border: none;
  padding: 12px;
  border-radius: 50%;
  cursor: pointer;
  z-index: 9998;
  font-size: 16px;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  backdrop-filter: blur(10px);
}

.debug-toggle-floating:hover {
  background: rgba(0, 86, 179, 0.9);
  transform: scale(1.1);
}

.debug-info-overlay {
  position: fixed;
  bottom: 10px;
  left: 10px;
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-family: monospace;
  z-index: 9997;
  pointer-events: none;
}
</style>
