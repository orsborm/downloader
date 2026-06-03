# Windows 便携版打包脚本
# 构建 Tauri 应用并打包为便携版 ZIP（零安装、零残留）
#
# 使用方法：
#   powershell -ExecutionPolicy Bypass -File scripts/build-portable.ps1
#
# 前置条件：
#   - Node.js 20+
#   - Rust toolchain (rustup)
#   - npm 依赖已安装 (npm ci)

param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipTests,
    [switch]$SkipFrontend
)

$ErrorActionPreference = "Stop"

$ROOT = Split-Path -Parent $PSScriptRoot
$DIST = Join-Path $ROOT "dist"
$SRC_TAURI = Join-Path $ROOT "src-tauri"
$VERSION = (Get-Content (Join-Path $ROOT "package.json") | ConvertFrom-Json).version

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  全协议下载器 - Windows 便携版打包" -ForegroundColor Cyan
Write-Host "  版本: $VERSION" -ForegroundColor Cyan
Write-Host "  目标: $Target" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 步骤 1: 前端构建
if (-not $SkipFrontend) {
    Write-Host "[1/4] 构建前端..." -ForegroundColor Yellow
    Push-Location $ROOT
    try {
        npm run build
        if ($LASTEXITCODE -ne 0) { throw "前端构建失败" }
        Write-Host "  前端构建成功" -ForegroundColor Green
    } finally {
        Pop-Location
    }
} else {
    Write-Host "[1/4] 跳过前端构建" -ForegroundColor DarkGray
}

# 步骤 2: 运行测试
if (-not $SkipTests) {
    Write-Host "[2/4] 运行测试..." -ForegroundColor Yellow
    Push-Location $ROOT
    try {
        npx vitest run
        if ($LASTEXITCODE -ne 0) { throw "测试失败" }
        Write-Host "  测试通过" -ForegroundColor Green
    } finally {
        Pop-Location
    }
} else {
    Write-Host "[2/4] 跳过测试" -ForegroundColor DarkGray
}

# 步骤 3: Tauri 构建
Write-Host "[3/4] 构建 Tauri 应用..." -ForegroundColor Yellow
Push-Location $ROOT
try {
    npx tauri build --target $Target
    if ($LASTEXITCODE -ne 0) { throw "Tauri 构建失败" }
    Write-Host "  Tauri 构建成功" -ForegroundColor Green
} finally {
    Pop-Location
}

# 步骤 4: 打包便携版
Write-Host "[4/4] 打包便携版..." -ForegroundColor Yellow

$BUNDLE_DIR = Join-Path $SRC_TAURI "target\$Target\release\bundle"
$NSIS_DIR = Join-Path $BUNDLE_DIR "nsis"
$PORTABLE_DIR = Join-Path $ROOT "release\portable"

# 创建便携版目录
if (Test-Path $PORTABLE_DIR) { Remove-Item $PORTABLE_DIR -Recurse -Force }
New-Item -ItemType Directory -Path $PORTABLE_DIR -Force | Out-Null

# 复制可执行文件
$EXE = Get-ChildItem -Path (Join-Path $SRC_TAURI "target\$Target\release") -Filter "*.exe" | Select-Object -First 1
if ($EXE) {
    Copy-Item $EXE.FullName $PORTABLE_DIR
    Write-Host "  复制: $($EXE.Name)" -ForegroundColor DarkGray
} else {
    throw "未找到可执行文件"
}

# 创建 data 目录（便携模式标识）
$DATA_DIR = Join-Path $PORTABLE_DIR "data"
New-Item -ItemType Directory -Path $DATA_DIR -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $DATA_DIR "logs") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $DATA_DIR "temp") -Force | Out-Null

# 创建默认配置文件
$DEFAULT_CONFIG = @"
# 全协议下载器配置文件
# 此文件在便携模式下位于程序目录/data/config.toml

[general]
language = "zh-CN"
theme = "system"
minimize_to_tray = true
close_to_tray = false
auto_start = false
font_size = 14

[download]
default_dir = "./downloads"
complete_dir = "./downloads/complete"
temp_dir = "./data/temp"
max_concurrent_tasks = 3
max_connections_per_task = 64
max_global_connections = 200
max_upload_speed = 0
max_download_speed = 0
auto_retry_count = 3
auto_retry_interval = 5

[connection]
bt_port = 6881
ed2k_port = 4661
http_port = 0
upnp = true
nat_pmp = true
proxy_type = "none"
connection_timeout = 30
read_timeout = 60

[notification]
task_complete = true
task_error = true
sound = true

[advanced]
log_level = "info"
log_max_size = 10
log_retain_days = 7
"@
Set-Content -Path (Join-Path $DATA_DIR "config.toml") -Value $DEFAULT_CONFIG -Encoding UTF8

# 创建 README
$README = @"
全协议下载器 v$VERSION - 便携版
================================

使用方法：
  1. 解压到任意目录
  2. 运行 Downloader.exe
  3. 所有数据保存在 data/ 目录下

特点：
  - 零安装：无需安装，解压即用
  - 零残留：删除目录即可完全卸载
  - 便携化：所有配置和数据在程序目录下

支持协议：
  - HTTP/HTTPS/FTP
  - BitTorrent (.torrent / magnet)
  - eDonkey2000 (ed2k)
  - HLS/DASH 流媒体

更新日志：见 CHANGELOG.md
"@
Set-Content -Path (Join-Path $PORTABLE_DIR "README.txt") -Value $README -Encoding UTF8

# 创建 ZIP
$ZIP_NAME = "Downloader_v${VERSION}_portable_x64.zip"
$ZIP_PATH = Join-Path $ROOT "release\$ZIP_NAME"
if (Test-Path $ZIP_PATH) { Remove-Item $ZIP_PATH -Force }
Compress-Archive -Path "$PORTABLE_DIR\*" -DestinationPath $ZIP_PATH -CompressionLevel Optimal

$SIZE = [math]::Round((Get-Item $ZIP_PATH).Length / 1MB, 2)
Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  打包完成!" -ForegroundColor Green
Write-Host "  便携版: release\$ZIP_NAME ($SIZE MB)" -ForegroundColor Green
Write-Host "  安装版: $NSIS_DIR\*.exe" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
