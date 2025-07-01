<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-25 18:05:32
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-07-02 06:39:23
 * @FilePath: \liuyao_desktop_tauri\README.md
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
# 六爻排盘与研究  桌面版

测试 GitHub 到 Gitee 的自动同步功能 - 测试时间：2024-03-27

基于 Tauri + Vue3 + TypeScript 开发的六爻桌面应用。

## 功能特点

- 🚀 基于 Tauri，性能高，体积小
- 🔄 支持多种代理设置（系统代理/手动代理）
- 🌐 内置网站访问支持
- 🛡️ 安全的跨域请求处理
- ~~💻 跨平台支持 (Windows, macOS, Linux)~~
- 💻 支持windows,(因macos , linux系统中调用webKit webKitGTK,不支持参数启动,无法实现独立代理管理,仅仅套个壳子有更简单的写法,两分钟的事情,不如另起项目不要工具栏及工具栏中各种功能,故取消.)

## 开发环境配置

### 系统要求

- [Node.js](https://nodejs.org/) (推荐 v18 或更高版本)
- [Rust](https://www.rust-lang.org/)
- [VS Code](https://code.visualstudio.com/) (推荐)

### 推荐的 VS Code 插件

- [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar)
- [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## 快速开始

1. 克隆项目
```bash
git clone [项目地址]
cd liuyao_desktop_tauri
```

2. 安装依赖
```bash
pnpm install
```

3. 开发模式运行
```bash
pnpm tauri dev
```

4. 构建应用
```bash
pnpm tauri build
```

## 更换内嵌网站

如果您想更换应用中内嵌的网站，请按以下步骤操作：

1. 打开 `src/components/MainContent.vue` 文件
2. 找到 iframe 相关配置代码
3. 修改 `targetUrl` 变量的值为您想要嵌入的网站地址
```vue
const targetUrl = ref('https://your-new-website.com');
```

注意事项：
- 确保目标网站允许被嵌入 iframe（X-Frame-Options 设置）
- 如果网站需要代理访问，请正确配置代理设置
- 建议在更换网站后全面测试功能，确保兼容性

## 代理设置说明

应用支持三种代理模式：

1. 禁用代理（默认）：直接访问，不使用任何代理
2. 系统代理：跟随系统代理设置
3. 手动代理：支持配置 HTTP、HTTPS、SOCKS5 代理

## 问题反馈

如果您在使用过程中遇到任何问题，欢迎提交 Issue 或 Pull Request。

## 许可证

[MIT License](LICENSE)
