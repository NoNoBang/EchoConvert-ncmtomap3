@echo off
REM NCM to MP3 Converter - Windows 快速开始脚本

echo.
echo 🎵 NCM to MP3 Converter - 快速开始 (Windows)
echo ============================================
echo.

REM 检查 Rust
echo 📦 检查 Rust 环境...
cargo --version > nul 2>&1
if errorlevel 1 (
    echo ❌ Rust 未安装。请从 https://win.rustup.rs/ 下载安装
    pause
    exit /b 1
)
echo ✅ Rust 已安装

REM 检查 Node.js
echo.
echo 📦 检查 Node.js 环境...
node --version > nul 2>&1
if errorlevel 1 (
    echo ❌ Node.js 未安装。请从 https://nodejs.org 安装
    pause
    exit /b 1
)
echo ✅ Node.js 已安装: %^node --version%

REM 检查 pnpm
echo.
echo 📦 检查 pnpm...
pnpm --version > nul 2>&1
if errorlevel 1 (
    echo ⏳ 安装 pnpm...
    npm install -g pnpm
)
echo ✅ pnpm 已安装

REM 安装依赖
echo.
echo 📦 安装项目依赖...
call pnpm install

if errorlevel 1 (
    echo ❌ 依赖安装失败
    pause
    exit /b 1
)

REM 启动开发服务器
echo.
echo 🚀 启动开发服务器...
echo 提示：首次编译 Rust 可能需要 5-10 分钟
echo.
call pnpm tauri dev

pause
