# 开发和构建指南

## 📋 前置要求

### 1. Rust 工具链（如果还未安装）

```bash
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Windows（PowerShell）
# 从 https://win.rustup.rs/ 下载并运行安装程序
```

**验证安装**：
```bash
rustc --version  # 应该看到类似 rustc 1.94.0
cargo --version  # 应该看到类似 cargo 1.94.0
```

### 2. Node.js 和 pnpm

```bash
# 检查是否已安装
node --version   # 需要 18+
npm --version

# 安装 pnpm（推荐）
npm install -g pnpm
pnpm --version
```

### 3. 系统依赖（macOS）

对于 Tauri 在 macOS 上的开发，可能需要：

```bash
# Xcode Command Line Tools（如果还未安装）
xcode-select --install
```

## 🚀 运行开发环境

### 步骤 1：进入项目目录

```bash
cd /Users/conghe/Documents/ncm2mp3
```

### 步骤 2：安装前端依赖

```bash
pnpm install
```

**可能的问题**：
- 如果出现权限错误，尝试 `sudo pnpm install --no-optional`
- 如果 npm 源太慢，改为淘宝镜像：`pnpm config set registry https://registry.npmmirror.com`

### 步骤 3：启动开发服务器

```bash
pnpm tauri dev
```

这会：
1. 启动 Vite 前端开发服务器（http://localhost:5173）
2. 编译 Rust 后端
3. 启动 Tauri 桌面应用窗口

**首次编译可能需要 5-10 分钟**（Rust 依赖下载和编译）

## 🔨 构建生产版本

### 编译为可执行程序

```bash
pnpm tauri build
```

**输出位置**：
- **macOS**: `src-tauri/target/release/bundle/macos/ncm2mp3.app`
- **Windows**: `src-tauri/target/release/bundle/msi/ncm2mp3_1.0.0_x64_en-US.msi`
- **Linux**: `src-tauri/target/release/bundle/deb/ncm2mp3_1.0.0_amd64.deb`

## 📝 项目代码说明

### 前端 (Vue 3)
- **src/App.vue** - 主UI组件，包含：
  - 拖拽上传区域
  - 文件列表显示
  - 转换进度和日志
  - 的按钮控制

### 后端 (Rust)
- **src-tauri/src/main.rs** - 应用入口，定义 Tauri commands
- **src-tauri/src/ncm.rs** - 核心逻辑：
  - `convert()` - 批量转换NCM文件
  - `parse_ncm()` - 解析NCM文件结构
  - `decrypt_aes()` - AES-128-ECB解密（用于密钥和元数据）
  - `decrypt_rc4()` - RC4流密码解密（用于音频数据）
  - `parse_metadata()` - 提取文件元数据

### 配置文件
- **src-tauri/tauri.conf.json** - Tauri应用配置（窗口大小、图标等）
- **src-tauri/Cargo.toml** - Rust依赖
- **vite.config.js** - Vite前端配置

## 🔑 核心算法说明

### NCM 文件格式结构

```
+--------+---------+-------+---------+--------+----------+
| 头部   | 密钥数据 | 元数据 | 预留字节 | 音频数据 |
| 8字节  | N字节   | M字节 | 9字节   | ...    |
+--------+---------+-------+---------+--------+----------+
  CTNFv2  [加密]    [加密]           [加密RC4]
```

### 解密流程

1. **读取 NCM 头**（8字节）
   - 验证格式：`CTNFv2` 或 `CTNFv1`

2. **读取密钥数据**
   - 读取 4 字节长度
   - 用 CORE_KEY 进行 AES-128-ECB 解密 → 获得 RC4 密钥

3. **读取元数据**
   - 读取 4 字节长度
   - 用 META_KEY 进行 AES-128-ECB 解密 → 获得 JSON 格式的元数据
   - 从 JSON 中提取：音频格式、比特率、时长等

4. **解密音频数据**
   - 用 RC4 密钥对音频段进行流密码解密

5. **输出文件**
   - 根据元数据中的格式（通常 mp3）写出文件

### 关键常数

```rust
CORE_KEY = b"hzHRDJ5fy5HIGaKS"  // 网易云音乐核心加密密钥
META_KEY = b"!@#$%^&*(^)_+-=[]{}|;',<>?"  // 元数据加密密钥
```

## 🐛 常见问题

### Q: 编译出错 "could not compile"
**A**: 
```bash
# 清除缓存重新编译
cargo clean
pnpm tauri dev
```

### Q: 前端页面显示不出来
**A**: 检查 Vite 开发服务器是否启动
```bash
# 查看端口 5173 是否在监听
lsof -i :5173
```

### Q: Windows 编译失败
**A**: 确保已安装 Visual Studio Build Tools（C++ 工具链）

### Q: macOS 出现代码签名问题
**A**: 
```bash
# 禁用代码签名检查（仅用于开发）
TAURI_SKIP_DEVSERVER_CHECK=true pnpm tauri dev
```

## 📚 参考资源

- [Tauri 官网文档](https://tauri.app/v1/guides/getting-started/prerequisites)
- [Vue 3 学习](https://vuejs.org/guide/introduction.html)
- [Rust Book](https://doc.rust-lang.org/book/)
- [参考项目](https://github.com/will-17173/tauri-ncm2mp3)

## 🔐 安全提示

- **本地处理**：所有转换在本地完成，不上传任何文件
- **开源审计**：完整代码可见，可自行审查
- **版权声明**：仅供学习和个人使用，请尊重音乐版权

---

祝你开发顺利！如有问题，查看输出日志 `src-tauri/target/release/` 或 GitHub Issues。
