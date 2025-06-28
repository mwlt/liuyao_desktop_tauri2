# GitHub-Gitee 自动同步完整配置指南

## 📋 当前状态
- ✅ SSH 密钥已生成
- ✅ 工作流文件已存在
- ✅ GitHub Token 已提供: `WM2AxvHtaQEBfkIYDWqhHvJI73Byk44Ww9ru`

## 🚀 立即执行的配置步骤

### 1. 配置 GitHub Repository Secrets

进入：`https://github.com/mwlt/liuyao_desktop_tauri2/settings/secrets/actions`

**添加以下 Secrets：**

#### A. GITEE_PRIVATE_KEY
```
-----BEGIN OPENSSH PRIVATE KEY-----
b3BlbnNzaC1rZXktdjEAAAAACmFlczI1Ni1jdHIAAAAGYmNyeXB0AAAAGAAAABA+uR7JgH
kOVnTlV+lAQxSNAAAAGAAAAAEAAAAzAAAAC3NzaC1lZDI1NTE5AAAAIG4rrK34OEoTPvjV
ziw7T2m1tDTtglKRLs0PkdQ82wEkAAAAkEnAOWwgqBfOKbTzvoatZqAkqensdnUle32a9p
aoh/L3sDpaEHzB7tsc8YsqzwXoMA4IcuAnQDUPq8kvgh5PqpP5waiaAi0PAj6aGW41EtiY
nDsZCAu5hYdPp6DFysQbtpqSeqM81d8XTpB5tVflpJO+P80B6zjdwNfIvsXk0wEFxpw4Wp
PP+094UX0uJTa9NA==
-----END OPENSSH PRIVATE KEY-----
```

#### B. PERSONAL_ACCESS_TOKEN
```
WM2AxvHtaQEBfkIYDWqhHvJI73Byk44Ww9ru
```

#### C. GITEE_PASSWORD
```
[需要你的 Gitee 访问令牌]
```

### 2. 配置 Gitee SSH 公钥

进入：`https://gitee.com/profile/sshkeys`

**添加公钥：**
```
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG4rrK34OEoTPvjVziw7T2m1tDTtglKRLs0PkdQ82wEk gitee-sync
```

### 3. 配置 GitHub Actions 权限

进入：`https://github.com/mwlt/liuyao_desktop_tauri2/settings/actions`

**设置：**
- ✅ "Read and write permissions"
- ✅ "Allow GitHub Actions to create and approve pull requests"

### 4. 创建 Gitee 访问令牌

1. 进入：`https://gitee.com/profile/personal_access_tokens`
2. 创建新令牌，选择权限：
   - `projects` - 仓库管理
   - `user_info` - 用户信息
3. 将令牌添加到 GitHub Secrets 作为 `GITEE_PASSWORD`

## 🧪 测试配置

配置完成后，运行测试： 