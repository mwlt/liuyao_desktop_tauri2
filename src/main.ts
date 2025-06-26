/*
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-25 18:05:32
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-25 21:18:52
 * @FilePath: \liuyao_desktop_tauri\src\main.ts
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
 */
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";

// 导入 Tauri API - 确保 __TAURI__ 全局对象可用
import '@tauri-apps/api/core';

// 添加 Naive UI 样式
import 'vfonts/Lato.css'
import 'vfonts/FiraCode.css'

import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
// 移除错误的样式导入，Naive UI 会自动处理样式

// 通用字体
const meta = document.createElement('meta')
meta.name = 'naive-ui-style'
document.head.appendChild(meta)

const app = createApp(App);
const pinia = createPinia();
pinia.use(piniaPluginPersistedstate)

app.use(pinia);
app.mount("#app");
