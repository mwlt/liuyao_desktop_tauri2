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

# 使用 Tauri 构建（而不是 cargo build）
Write-Host "🏗️ 使用 Tauri 构建应用..."
Set-Location src-tauri
cargo tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tauri 构建失败"
    exit 1
}

# 创建便携版目录
$portableDir = "target/portable/liuyao-desktop-portable-v$version"
Write-Host "📦 创建便携版目录: $portableDir"
New-Item -Path $portableDir -ItemType Directory -Force

# 复制 Tauri 构建的可执行文件
Write-Host "📝 复制文件..."
Copy-Item "target/release/liuyao_desktop_tauri.exe" -Destination "$portableDir/"

# 注意：icons 文件夹已嵌入到 exe 中，无需复制

# 创建便携版说明
$readmeContent = @"
六爻排盘与研究 便携版 v$version

使用说明：
1. 本程序为免安装版本，无需安装即可使用
2. 直接双击 liuyao_desktop_tauri.exe 即可运行程序
3. 首次运行可能需要允许防火墙访问

注意事项：
1. 请不要删除任何程序文件
2. 如需完全卸载，直接删除整个文件夹即可
3. 如遇问题，请访问 https://github.com/mwlt/liuyao-desktop 寻求帮助

祝使用愉快！
"@

$readmeContent | Out-File -FilePath "$portableDir/说明.txt" -Encoding utf8

# 创建ZIP包
Write-Host "📦 创建ZIP包..."
$zipPath = "target/portable/liuyao-desktop-portable-v$version.zip"
Compress-Archive -Path $portableDir -DestinationPath $zipPath -Force

Write-Host "✅ 便携版构建完成！"
Write-Host "📂 文件位置：$zipPath" 