<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-23 00:19:54
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-28 00:59:15
 * @FilePath: \liuyao_desktop2\frontend\src\App.vue
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
<script setup lang="ts">
  import { onMounted } from 'vue'
  import NavBar from './components/NavBar.vue';
  import MainContent from './components/MainContent.vue';
  import { useProxyStore } from './store/useProxyStore';

  const proxyStore = useProxyStore();
  onMounted(async () => {
    
    
    // 延迟一点时间确保Tauri完全初始化
  
       
      proxyStore.initialize();
    
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

  