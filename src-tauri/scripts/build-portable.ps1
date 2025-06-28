# 设置环境变量以构建便携版
$env:TAURI_BUNDLE_PORTABLE = "true"

# 获取当前目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $scriptDir/..

# 获取版本号
$version = (Get-Content -Raw -Path "tauri.conf.json" | ConvertFrom-Json).version

# 清理之前的构建
Write-Host "🧹 清理之前的构建..."
Remove-Item -Path "target/release" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "target/portable" -Recurse -ErrorAction SilentlyContinue

# 构建前端
Write-Host "🏗️ 构建前端..."
Set-Location ..
pnpm build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 前端构建失败"
    exit 1
}

# 构建后端
Write-Host "🏗️ 构建后端..."
Set-Location src-tauri
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 后端构建失败"
    exit 1
}

# 创建便携版目录
$portableDir = "target/portable/六爻排盘与研究-便携版-v$version"
Write-Host "📦 创建便携版目录: $portableDir"
New-Item -Path $portableDir -ItemType Directory -Force

# 复制必要文件
Write-Host "📝 复制文件..."
Copy-Item "target/release/liuyao_desktop_tauri.exe" -Destination "$portableDir/"
Copy-Item "../dist" -Destination "$portableDir/" -Recurse
Copy-Item "icons" -Destination "$portableDir/" -Recurse

# 创建启动脚本
@"
@echo off
echo 正在启动六爻排盘与研究...
start liuyao_desktop_tauri.exe
"@ | Out-File -FilePath "$portableDir/启动.bat" -Encoding ascii

# 创建便携版说明
@"
六爻排盘与研究 便携版 v$version

使用说明：
1. 本程序为免安装版本，无需安装即可使用
2. 双击"启动.bat"即可运行程序
3. 所有数据都保存在程序目录下，方便备份和迁移
4. 可以直接复制整个文件夹到其他位置使用
5. 首次运行可能需要允许防火墙访问

注意事项：
1. 请不要删除任何程序文件
2. 如需完全卸载，直接删除整个文件夹即可
3. 如遇问题，请访问 https://github.com/mwlt/liuyao-desktop 寻求帮助

祝使用愉快！
"@ | Out-File -FilePath "$portableDir/说明.txt" -Encoding utf8

# 创建ZIP包
Write-Host "📦 创建ZIP包..."
$zipPath = "target/portable/六爻排盘与研究-便携版-v$version.zip"
Compress-Archive -Path $portableDir -DestinationPath $zipPath -Force

Write-Host "✅ 便携版构建完成！"
Write-Host "📂 文件位置：$zipPath" 