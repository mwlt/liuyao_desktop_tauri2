cc方案：Tauri 2.6 桌面应用智能分流代理开发方案
一、方案概述
目标：
让桌面应用在操作系统设置了全局代理的情况下，依然能无障碍访问 www.core333.com 及 file.core333.com（七牛云），并且所有流量都能被后端 Rust 代理服务器智能分流和掌控，前端无需大改，仅在现有基础上增强功能。
核心思路：
启动 Tauri WebView 时设置全局代理参数，指向本地 Rust 代理服务器。
Rust 代理服务器根据规则（如 core333.com 直连，其他走系统代理）智能分流。
前端 iframe、fetch、XHR、图片、视频等请求全部自动走本地代理，无需特殊处理。
前端代理设置 UI 只需增强，不大改界面。
二、实施步骤
1. Rust 后端开发
1.1 集成本地 HTTP/HTTPS 代理服务器库
推荐使用 tokio + hyper 或 mitmproxy-rs 实现。
支持 HTTP/HTTPS 代理，监听本地端口（如 127.0.0.1:8080）。
1.2 实现智能分流逻辑
维护白名单（如 core333.com、file.core333.com），这些域名直连。
其他域名可按需走系统代理或自定义代理。
支持 no_proxy 规则。
1.3 暴露代理配置 API
提供获取/设置代理规则、白名单、no_proxy、当前状态等接口，供前端调用。
提供系统代理信息读取接口。
1.4 启动时自动启动本地代理服务器
确保 WebView 启动前代理服务已监听。
1.5 跨平台兼容性处理
Windows/Linux/macOS 均需支持，macOS 可提示用户需手动设置系统代理。
2. Tauri 配置
2.1 设置 WebView 启动参数
在 tauri.conf.json 或 Rust 端设置 --proxy-server=127.0.0.1:8080。
Windows/Linux 可直接生效，macOS 需提示用户设置系统代理。
2.2 安全性与权限
限制本地代理仅监听 127.0.0.1，防止外部访问。
处理好证书信任问题（如需支持 HTTPS）。
3. 前端 Vue 增强
3.1 代理设置 UI 增强
保持现有 ProxySettingsPopover、Settings、NavBar 结构不变，仅增强功能。
支持切换“禁用代理/系统代理/手动代理/智能分流”。
支持显示当前代理状态、系统代理信息、连接测试。
3.2 与后端通信
通过 window.go.main.App.* 方法与后端同步代理配置。
监听后端事件，实时刷新代理状态。
3.3 主内容区 iframe
保持 iframe src 指向 www.core333.com，不用特殊处理，所有流量自动走本地代理。
3.4 store 逻辑优化
增强 useProxyStore.ts，支持更多代理类型和状态同步。
4. 测试与优化
4.1 跨平台测试
Windows、Linux、macOS 全面测试代理分流效果。
特别关注 macOS 下的兼容性和用户提示。
4.2 性能与稳定性
压测本地代理服务器，优化转发性能。
增强错误处理和日志记录。
4.3 用户体验
代理切换、连接测试、状态展示流畅无卡顿。
代理异常时有明确提示。
三、目录

``` 
src/
  components/
    NavBar.vue
    ProxySettingsPopover.vue
    Settings.vue
    MainContent.vue
  store/
    useProxyStore.ts
  types/
    wails.d.ts
src-tauri/
  src/
    main.rs
    proxy_server.rs   # 新增，代理服务实现
  ...
tauri.conf.json
```

四、注意事项
macOS 下 WKWebView 不支持代理参数，需提示用户设置系统代理或采用内容重写方案。
HTTPS 代理需处理证书信任问题，可引导用户安装本地 CA 证书。
代理服务器需支持多协议（HTTP/HTTPS/SOCKS5）和 no_proxy 例外。