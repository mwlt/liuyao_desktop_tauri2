@echo off
REM 进入脚本所在目录（假设和 build-portable.ps1 在同一目录）
cd /d "%~dp0"

REM 调用 PowerShell 执行 build-portable.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File ".\src-tauri\scripts\build-portable.ps1"

pause