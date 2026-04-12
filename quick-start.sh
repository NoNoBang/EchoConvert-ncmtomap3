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

echo ""frv4|<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>歌单解析助手 - 极简工具版</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdnjs.cloudflare.com/ajax/libs/lucide/0.263.0/lucide.min.js"></script>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=PingFang+SC:wght@400;500;600&display=swap');
        
        :root {
            --quark-blue: #3267ff;
            --quark-bg: #ffffff;
            --quark-gray: #f7f8fa;
        }

        body { 
            font-family: 'PingFang SC', 'Microsoft YaHei', sans-serif; 
            background-color: var(--quark-bg);
            color: #1a1a1a;
            overflow: hidden;
        }

        .sidebar-item {
            position: relative;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            width: 50px;
            height: 50px;
            margin: 12px auto;
            border-radius: 12px;
            color: #707277;
            transition: all 0.2s;
            cursor: pointer;
        }
        .sidebar-item.active { color: var(--quark-blue); }
        .sidebar-item.active::before {
            content: ''; position: absolute; left: -15px; height: 20px; width: 4px; background: var(--quark-blue); border-radius: 0 4px 4px 0;
        }

        .search-wrapper { transition: all 0.6s cubic-bezier(0.16, 1, 0.3, 1); max-width: 800px; width: 100%; }
        .search-initial { transform: translateY(22vh); }
        .search-collapsed { transform: translateY(0); margin-bottom: 24px; max-width: 100%; }

        .search-box { background: #fff; border-radius: 24px; box-shadow: 0 4px 30px rgba(0,0,0,0.06); border: 1px solid #f0f0f0; padding: 8px; }

        .custom-scroll::-webkit-scrollbar { width: 5px; }
        .custom-scroll::-webkit-scrollbar-track { background: transparent; }
        .custom-scroll::-webkit-scrollbar-thumb { background: #e2e8f0; border-radius: 10px; }
        
        .tab-btn { position: relative; padding: 8px 16px; font-size: 14px; transition: all 0.2s; cursor: pointer; color: #707277; }
        .tab-btn.active { color: #1a1a1a; font-weight: 600; }
        .tab-btn.active::after {
            content: ''; position: absolute; bottom: 0; left: 50%; transform: translateX(-50%); width: 20px; height: 3px; background: var(--quark-blue); border-radius: 2px;
        }
    </style>
</head>
<body class="h-screen flex">

    <!-- 侧边栏 -->
    <aside class="w-20 bg-white border-r border-gray-50 flex flex-col py-8 items-center z-20">
        <div class="mb-12">
            <div class="w-11 h-11 bg-blue-600 rounded-2xl flex items-center justify-center text-white shadow-xl shadow-blue-100">
                <i data-lucide="music-2" class="w-6 h-6"></i>
            </div>
        </div>
        <nav class="space-y-4">
            <div data-tab="home" class="sidebar-item active js-nav-btn">
                <i data-lucide="home" class="w-6 h-6"></i>
                <span class="text-[11px] mt-1 font-medium">主页</span>
            </div>
            <div data-tab="transfer" class="sidebar-item relative js-nav-btn">
                <i data-lucide="arrow-down-to-line" class="w-6 h-6"></i>
                <span class="text-[11px] mt-1 font-medium">传输</span>
                <span id="globalBadge" class="absolute top-2 right-2 w-4 h-4 bg-blue-500 text-white text-[9px] rounded-full flex items-center justify-center hidden">0</span>
            </div>
        </nav>
    </aside>

    <main class="flex-1 flex flex-col min-w-0 bg-white relative">
        
        <!-- 主页 -->
        <section id="view-home" class="view-content flex-1 flex flex-col px-10 py-6 overflow-hidden">
            <div id="searchContainer" class="search-wrapper mx-auto">
                <div id="welcomeText" class="text-center mb-10 flex flex-col items-center">
                    <div class="text-4xl font-bold text-gray-900">你好，我是歌单助手</div>
                    <p class="text-gray-400 mt-3 text-lg">粘贴链接，一键获取您的云端乐库</p>
                </div>
                <div class="search-box">
                    <div class="p-4 text-lg">
                        <input type="text" id="urlInput" placeholder="在此输入歌单网址..." class="w-full outline-none">
                    </div>
                    <div class="flex items-center justify-between px-3 pb-2 pt-1">
                        <div class="px-3 py-1.5 bg-blue-50 text-blue-600 rounded-xl text-sm font-medium flex items-center gap-2">
                            <i data-lucide="link-2" class="w-4 h-4"></i>自动识别
                        </div>
                        <button id="parseBtn" class="bg-blue-600 text-white px-6 py-2.5 rounded-xl text-sm font-bold flex items-center gap-2 transition-all hover:translate-y-[-1px]">
                            <span>开始解析</span><i data-lucide="arrow-right" class="w-4 h-4"></i>
                        </button>
                    </div>
                </div>
            </div>

            <!-- 解析列表 -->
            <div id="resultArea" class="hidden flex-1 flex flex-col mt-4 min-h-0">
                <div class="flex items-center justify-between mb-6 px-4">
                    <div class="flex items-center gap-4">
                        <h2 class="font-bold text-lg text-gray-800">解析结果</h2>
                        <span class="text-xs text-gray-400" id="songStat">已选择 0 项</span>
                    </div>
                    <button id="batchDownloadBtn" disabled class="bg-blue-600 text-white px-6 py-2 rounded-xl text-sm font-bold disabled:opacity-30">批量下载</button>
                </div>
                <div class="flex-1 overflow-auto custom-scroll border border-gray-50 rounded-2xl">
                    <table class="w-full">
                        <thead class="sticky top-0 bg-white border-b border-gray-50 z-10 text-[11px] text-gray-400 uppercase">
                            <tr>
                                <th class="px-6 py-4 w-12"><input type="checkbox" id="masterCheckbox" class="w-4 h-4 accent-blue-600"></th>
                                <th class="px-6 py-4 text-left">文件名</th>
                                <th class="px-6 py-4 text-left">作者</th>
                                <th class="px-6 py-4 text-left">类型</th>
                                <th class="px-6 py-4 text-left">大小</th>
                                <th class="px-6 py-4 w-12"></th>
                            </tr>
                        </thead>
                        <tbody id="songTbody" class="divide-y divide-gray-50 text-sm"></tbody>
                    </table>
                </div>
            </div>
        </section>

        <!-- 传输界面 -->
        <section id="view-transfer" class="view-content hidden flex-1 flex flex-col overflow-hidden absolute inset-0 bg-white z-10">
            <!-- 顶部标题 -->
            <div class="px-10 pt-8 pb-4 flex items-center justify-between border-b border-gray-50">
                <div class="flex items-center gap-4">
                    <h2 class="text-xl font-bold">传输</h2>
                    <div class="flex gap-4 ml-6">
                        <div data-subtab="downloading" class="tab-btn active js-sub-tab">正在下载 <span id="countDownloading" class="ml-1 opacity-60">0</span></div>
                        <div data-subtab="completed" class="tab-btn js-sub-tab">已完成 <span id="countCompleted" class="ml-1 opacity-60">0</span></div>
                    </div>
                </div>
                <button id="clearAllBtn" class="flex items-center gap-2 text-xs text-gray-400 hover:text-red-500">
                    <i data-lucide="trash-2" class="w-3.5 h-3.5"></i>清空全部记录
                </button>
            </div>

            <!-- 列表容器 -->
            <div class="flex-1 overflow-auto custom-scroll px-10 py-6">
                <!-- 正在下载列表 -->
                <div id="list-downloading" class="space-y-4"></div>
                <!-- 已完成列表 -->
                <div id="list-completed" class="hidden space-y-1"></div>
            </div>
        </section>

    </main>

    <div id="toast" class="fixed bottom-10 left-1/2 -translate-x-1/2 bg-gray-900 text-white px-6 py-3 rounded-2xl shadow-2xl transform translate-y-32 transition-all duration-300 z-50 text-sm font-medium"></div>

    <script>
        // 等待页面及外部脚本加载完成
        window.addEventListener('load', () => {
            const MOCK_DATA = [
                { id: 1, title: "夜曲 - 周杰伦.mp3", artist: "周杰伦", type: "MP3", size: "4.5MB" },
                { id: 2, title: "晴天 - 周杰伦.wav", artist: "周杰伦", type: "WAV", size: "38.2MB" },
                { id: 3, title: "消愁 - 毛不易.flac", artist: "毛不易", type: "FLAC", size: "29.8MB" },
                { id: 4, title: "稻香 - 周杰伦.mp3", artist: "周杰伦", type: "MP3", size: "5.1MB" },
                { id: 5, title: "玫瑰少年 - 蔡依林.m4a", artist: "蔡依林", type: "M4A", size: "8.4MB" }
            ];

            let downloadingTasks = [];
            let completedTasks = [];
            let selectedIds = new Set();
            
            // 初始化图标
            if (typeof lucide !== 'undefined') {
                lucide.createIcons();
            }

            // 基础导航
            document.querySelectorAll('.js-nav-btn').forEach(btn => {
                btn.onclick = function() {
                    const target = this.dataset.tab;
                    document.querySelectorAll('.sidebar-item').forEach(b => b.classList.remove('active'));
                    this.classList.add('active');
                    document.querySelectorAll('.view-content').forEach(v => v.classList.add('hidden'));
                    document.getElementById(`view-${target}`).classList.remove('hidden');
                };
            });

            // 传输内部子页签切换
            document.querySelectorAll('.js-sub-tab').forEach(btn => {
                btn.onclick = function() {
                    const target = this.dataset.subtab;
                    document.querySelectorAll('.js-sub-tab').forEach(b => b.classList.remove('active'));
                    this.classList.add('active');
                    document.getElementById('list-downloading').classList.toggle('hidden', target !== 'downloading');
                    document.getElementById('list-completed').classList.toggle('hidden', target !== 'completed');
                };
            });

            // 解析逻辑
            const parseBtn = document.getElementById('parseBtn');
            parseBtn.onclick = function() {
                const input = document.getElementById('urlInput').value.trim();
                if(!input) return showToast("请输入链接");
                this.disabled = true;
                this.innerHTML = `<span class="animate-pulse">正在解析...</span>`;
                setTimeout(() => {
                    document.getElementById('searchContainer').classList.replace('search-initial', 'search-collapsed');
                    document.getElementById('welcomeText').classList.add('hidden');
                    document.getElementById('resultArea').classList.remove('hidden');
                    renderList();
                    this.disabled = false;
                    this.innerHTML = `<span>解析完成</span><i data-lucide="check" class="w-4 h-4"></i>`;
                    if (typeof lucide !== 'undefined') lucide.createIcons();
                }, 800);
            };

            function renderList() {
                document.getElementById('songTbody').innerHTML = MOCK_DATA.map(item => `
                    <tr class="hover:bg-blue-50/30 transition-colors group">
                        <td class="px-6 py-5 text-center"><input type="checkbox" class="song-cb w-4 h-4 accent-blue-600" data-id="${item.id}"></td>
                        <td class="px-6 py-5 flex items-center gap-3">
                            <div class="w-9 h-9 bg-blue-50 text-blue-500 rounded-xl flex items-center justify-center border border-blue-100"><i data-lucide="music" class="w-4 h-4"></i></div>
                            <span class="font-medium text-gray-800">${item.title}</span>
                        </td>
                        <td class="px-6 py-5 text-gray-500">${item.artist}</td>
                        <td class="px-6 py-5 text-gray-400 font-mono text-xs">${item.type}</td>
                        <td class="px-6 py-5 text-gray-400 text-xs">${item.size}</td>
                        <td class="px-6 py-5 text-right"><button onclick="startDownload(${item.id})" class="text-gray-300 hover:text-blue-600"><i data-lucide="download" class="w-5 h-5"></i></button></td>
                    </tr>
                `).join('');
                if (typeof lucide !== 'undefined') lucide.createIcons();
                bindCheckboxes();
            }

            function bindCheckboxes() {
                document.querySelectorAll('.song-cb').forEach(cb => {
                    cb.onchange = (e) => {
                        const id = parseInt(e.target.dataset.id);
                        if(e.target.checked) selectedIds.add(id); else selectedIds.delete(id);
                        updateStat();
                    };
                });
            }

            function updateStat() {
                document.getElementById('songStat').innerText = `已选择 ${selectedIds.size} 项`;
                document.getElementById('batchDownloadBtn').disabled = selectedIds.size === 0;
            }

            // 下载逻辑 (挂载到全局以便 inline onclick 调用)
            window.startDownload = function(id) {
                const item = MOCK_DATA.find(i => i.id === id);
                const taskId = Date.now() + id;
                const now = new Date();
                const timeStr = `${now.getFullYear()}/${(now.getMonth()+1).toString().padStart(2,'0')}/${now.getDate()} ${now.getHours()}:${now.getMinutes()}`;
                
                downloadingTasks.push({ ...item, taskId, progress: 0, time: timeStr });
                showToast(`已添加: ${item.title}`);
                updateTransferUI();

                const interval = setInterval(() => {
                    const task = downloadingTasks.find(t => t.taskId === taskId);
                    if(!task) return clearInterval(interval);
                    task.progress += Math.random() * 25;
                    if(task.progress >= 100) {
                        task.progress = 100;
                        completedTasks.unshift(task);
                        downloadingTasks = downloadingTasks.filter(t => t.taskId !== taskId);
                        clearInterval(interval);
                    }
                    updateTransferUI();
                }, 600);
            };

            function updateTransferUI() {
                const downList = document.getElementById('list-downloading');
                const compList = document.getElementById('list-completed');
                
                document.getElementById('countDownloading').innerText = downloadingTasks.length;
                document.getElementById('countCompleted').innerText = completedTasks.length;
                const badge = document.getElementById('globalBadge');
                badge.innerText = downloadingTasks.length;
                badge.classList.toggle('hidden', downloadingTasks.length === 0);

                downList.innerHTML = downloadingTasks.map(t => `
                    <div class="bg-white p-5 rounded-2xl border border-gray-100 flex flex-col gap-3 shadow-sm">
                        <div class="flex justify-between items-center">
                            <div class="flex items-center gap-3">
                                <i data-lucide="music-2" class="w-5 h-5 text-blue-500"></i>
                                <span class="font-medium text-gray-700">${t.title}</span>
                            </div>
                            <span class="text-sm font-bold text-blue-600">${Math.floor(t.progress)}%</span>
                        </div>
                        <div class="w-full bg-gray-50 h-1.5 rounded-full overflow-hidden">
                            <div class="bg-blue-600 h-full transition-all duration-300" style="width: ${t.progress}%"></div>
                        </div>
                    </div>
                `).join('') || `<div class="py-20 text-center text-gray-300 text-sm italic">暂无进行中的下载</div>`;

                compList.innerHTML = completedTasks.map(t => `
                    <div class="flex items-center justify-between p-4 hover:bg-gray-50 rounded-xl group transition-all">
                        <div class="flex items-center gap-4 flex-1">
                            <div class="w-10 h-10 bg-gray-50 rounded-lg flex items-center justify-center border border-gray-100"><i data-lucide="file-audio" class="w-5 h-5 text-red-400"></i></div>
                            <div class="flex flex-col">
                                <span class="text-sm font-medium text-gray-800">${t.title}</span>
                                <span class="text-[10px] text-gray-400 mt-0.5 uppercase tracking-wider">${t.type} · ${t.size}</span>
                            </div>
                        </div>
                        <div class="text-[12px] text-gray-300 w-40 text-left hidden md:block">${t.time}</div>
                        <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button class="p-2 hover:bg-white rounded-lg text-gray-400"><i data-lucide="folder-open" class="w-4 h-4"></i></button>
                            <button class="p-2 hover:bg-white rounded-lg text-gray-400 hover:text-red-500"><i data-lucide="trash" class="w-4 h-4"></i></button>
                        </div>
                    </div>
                `).join('') || `<div class="py-20 text-center text-gray-300 text-sm italic">暂无历史记录</div>`;
                
                if (typeof lucide !== 'undefined') lucide.createIcons();
            }

            function showToast(msg) {
                const t = document.getElementById('toast');
                t.innerText = msg;
                t.classList.replace('translate-y-32', 'translate-y-0');
                setTimeout(() => t.classList.replace('translate-y-0', 'translate-y-32'), 2500);
            }

            document.getElementById('masterCheckbox').onchange = (e) => {
                document.querySelectorAll('.song-cb').forEach(cb => {
                    cb.checked = e.target.checked;
                    const id = parseInt(cb.dataset.id);
                    if(e.target.checked) selectedIds.add(id); else selectedIds.delete(id);
                });
                updateStat();
            };

            document.getElementById('batchDownloadBtn').onclick = () => {
                selectedIds.forEach(id => window.startDownload(id));
                selectedIds.clear();
                document.getElementById('masterCheckbox').checked = false;
                updateStat();
            };

            document.getElementById('clearAllBtn').onclick = () => {
                completedTasks = [];
                updateTransferUI();
                showToast("记录已清空");
            };
        });
    </script>
</body>
</html>aqz1
echo "✅ 开发服务器已启动！"=】|
echo "🌐 前端: http://localhost:5173"
echo "🔧 Rust 后端: src-tauri/src/"
