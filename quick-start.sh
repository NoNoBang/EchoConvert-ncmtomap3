·副乳#!/bin/bash

# NCM to MP3 Converter - 快速开始脚本
# 这个脚本会自动安装和启动项目

set -e

echo "🎵 NCM to MP3 Converter - 快速开始"
echo "=================================="
echo ""

# 检查 Rust
echo "📦 检查 Rust 环境..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust 未安装。正在安装..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✅ Rust 已安装"
else
    echo "✅ Rust 已安装: $(rustc --version)"
fi

# 检查 Node.js
echo ""
echo "📦 检查 Node.js 环境..."
if ! command -v node &> /dev/null; then
    echo "❌ Node.js 未安装。请从 https://nodejs.org 安装"
    exit 1
else
    echo "✅ Node.js 已安装: $(node --version)"
fi

# 检查 pnpm
echo ""
echo "📦 检查 pnpm..."
if ! command -v pnpm &> /dev/null; then
    echo "⏳ 安装 pnpm..."
    npm install -g pnpm
fi
echo "✅ pnpm 已安装: $(pnpm --version)"

# 进入项目目录
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

# 安装依赖
echo ""
echo "📦 安装项目依赖..."
pnpm install

# 启动开发服务器
echo ""
echo "🚀 启动开发服务器..."
echo "提示：首次编译 Rust 可能需要 5-10 分钟"
echo ""
pnpm tauri dev

echo ""frv4
echo "✅ 开发服务器已启动！"=】|
echo "🌐 前端: http://localhost:5173"
echo "🔧 Rust 后端: src-tauri/src/"
