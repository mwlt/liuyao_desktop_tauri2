# 设置环境变量以构建便携版
$env:TAURI_BUNDLE_PORTABLE = "true"

# 获取项目根目录
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = (Get-Item $scriptDir).Parent.Parent.FullName
$tauriDir = Join-Path $projectRoot "src-tauri"

Write-Host "📌 项目根目录: $projectRoot"
Write-Host "📌 Tauri 目录: $tauriDir"

# 获取版本号
$tauriConfigPath = Join-Path $tauriDir "tauri.conf.json"
$version = (Get-Content -Raw -Path $tauriConfigPath | ConvertFrom-Json).version
Write-Host "📌 当前版本: $version"

# 清理之前的构建
Write-Host "🧹 清理之前的构建..."
Remove-Item -Path (Join-Path $tauriDir "target/release") -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path (Join-Path $tauriDir "target/portable") -Recurse -ErrorAction SilentlyContinue

# 构建前端
Write-Host "🏗️ 构建前端..."
Set-Location $projectRoot
Write-Host "📌 当前目录: $(Get-Location)"
pnpm build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 前端构建失败"
    exit 1
}

# 使用 Tauri 构建
Write-Host "🏗️ 使用 Tauri 构建应用..."
Set-Location $projectRoot  # CI 环境需要在项目根目录
Write-Host "📌 当前目录: $(Get-Location)"

# 确保目标目录存在
$targetDir = Join-Path $tauriDir "target"
if (-not (Test-Path $targetDir)) {
    New-Item -Path $targetDir -ItemType Directory -Force
    Write-Host "📁 创建目标目录: $targetDir"
}

# CI 环境使用 pnpm tauri build
pnpm tauri build
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Tauri 构建失败"
    Write-Host "📋 检查日志文件..."
    $buildLog = Join-Path $targetDir "debug/build.log"
    if (Test-Path $buildLog) {
        Write-Host "📝 构建日志内容:"
        Get-Content $buildLog | ForEach-Object { Write-Host "   $_" }
    }
    exit 1
}

# 创建便携版目录
$portableDir = Join-Path $tauriDir "target/portable/liuyao-desktop-portable-v$version"
Write-Host "📦 创建便携版目录: $portableDir"
New-Item -Path $portableDir -ItemType Directory -Force

# 复制构建的可执行文件
$exePath = Join-Path $tauriDir "target/release/liuyao_desktop_tauri.exe"
Write-Host "📝 复制可执行文件: $exePath"
if (Test-Path $exePath) {
    Copy-Item $exePath -Destination $portableDir/
} else {
    Write-Host "❌ 错误：找不到可执行文件: $exePath"
    exit 1
}

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
$zipPath = Join-Path $tauriDir "target/portable/liuyao-desktop-portable-v$version.zip"
Compress-Archive -Path $portableDir -DestinationPath $zipPath -Force

Write-Host "✅ 便携版构建完成！"
Write-Host "📂 文件位置：$zipPath" 