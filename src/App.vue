<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-23 00:19:54
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-25 21:45:20
 * @FilePath: \liuyao_desktop2\frontend\src\App.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<script setup lang="ts">
  import { ref, onMounted, onUnmounted } from 'vue'
  import NavBar from './components/NavBar.vue';
  import MainContent from './components/MainContent.vue';
  import { useProxyStore } from './store/useProxyStore';

  const proxyStore = useProxyStore();
  onMounted(async () => {
    console.log('App.vue mounted');
    console.log('window.__TAURI__ available:', !!(window as any).__TAURI__);
    console.log('window.__TAURI__.invoke available:', !!((window as any).__TAURI__?.invoke));
    
    // 延迟一点时间确保Tauri完全初始化
    setTimeout(() => {
      console.log('开始初始化本地代理端口...');
      proxyStore.initLocalProxyPort();
    }, 1000);
  });
  </script>
  
  <template>
    <div id="app-container">
      <NavBar   />
      <MainContent   />
    </div>
  </template>
  
  <style scoped>
  #app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }
  </style>

  