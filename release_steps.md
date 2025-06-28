<!--
 * @Author: mwlt_sanodia mwlt@163.com
 * @Date: 2025-06-28 20:35:08
 * @LastEditors: mwlt_sanodia mwlt@163.com
 * @LastEditTime: 2025-06-29 07:34:50
 * @FilePath: \liuyao_desktop_tauri\release_steps.md
 * @Description: 这是默认设置,请设置`customMade`, 打开koroFileHeader查看配置 进行设置: https://github.com/OBKoro1/koro1FileHeader/wiki/%E9%85%8D%E7%BD%AE
-->
# GitHub Release 发布步骤

## 1. 版本号更新
需要同时更新以下文件中的版本号：

### package.json
```json
{
  "name": "liuyao_desktop_tauri",
  "private": true,
  "version": "2.6.1",  // 更新此处
  ...
}
```

### src-tauri/tauri.conf.json
```json
{
  "productName": "六爻排盘与研究",
  "version": "2.6.1",  // 更新此处
  ...
}
```

## 2. 构建配置优化
在 `src-tauri/tauri.conf.json` 中添加 Linux 打包配置：
```json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [...],
    "windows": {
      "wix": {
        "language": "zh-CN"
      }
    },
    "linux": {
      "deb": {
        "files": {
          "/usr/bin/liuyao-desktop": "bin/liuyao-desktop"
        }
      }
    }
  }
}
```

## 3. 发布流程

### 3.1 提交更改
```bash
git add package.json src-tauri/tauri.conf.json
git commit -m "chore: update version to 2.6.1 and optimize build config"
git push liuyao_desktop_tauri2 main
```

### 3.2 标签管理
如果需要重新创建标签：
```bash
# 删除远程标签
git push liuyao_desktop_tauri2 :v2.6.1

# 删除本地标签
git tag -d v2.6.1

# 创建新标签
git tag v2.6.1

# 推送标签
git push liuyao_desktop_tauri2 v2.6.1
```

### 3.3 Release 检查清单
1. 确保没有正在运行的 workflow（如有，需要在 GitHub Actions 页面取消）
2. 检查 Release 页面，确保没有同名文件
3. 等待新的构建完成
4. 验证所有平台的构建文件是否正确生成
5. 发布 Release（从草稿状态改为正式发布）

## 4. 故障排除
如果遇到 "already exists" 错误：
1. 删除 GitHub 上的草稿 Release
2. 重新执行标签管理步骤
3. 等待新的构建完成

## 5. 文件命名规范
构建后会生成以下格式的文件：
- Windows: `六爻排盘与研究_${version}_x64.msi`
- macOS: `六爻排盘与研究_${version}_aarch64.dmg`
- Linux: 
  - `六爻排盘与研究_${version}_amd64.deb`
  - `六爻排盘与研究_${version}_amd64.AppImage`
  - `六爻排盘与研究-${version}-1.x86_64.rpm` 