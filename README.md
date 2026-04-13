# EchoConvert

轻量的 WYY 歌单下载桌面工具（Tauri + Vue + Rust）。

## 界面预览

![首页预览](./readmesrc/image.png)

![解析结果预览](./readmesrc/image1.png)

## 主要功能

1. WYY 歌单一键导入  
输入歌单链接或歌单 ID，自动解析歌曲列表。

2. 下载 MP3 到本地  
支持单曲下载和批量下载，传输页可查看实时进度与历史记录。

## 下载地址

- Releases: [https://github.com/NoNoBang/EchoConvert/releases](https://github.com/NoNoBang/EchoConvert/releases)
- macOS：下载 `.dmg`（或 `.app.zip`）
- Windows：下载 `.msi`（或 `.exe`）

## 自动发布（GitHub Actions）

仓库使用 `tag` 自动发布流程：推送 `v*` 标签后，自动构建 macOS/Windows 安装包并上传到 Releases。

```bash
git add .
git commit -m "release: v1.0.0"
git push origin main

git tag v1.0.0
git push origin v1.0.0
```

工作流文件：`.github/workflows/release.yml`

## 本地开发

```bash
pnpm install
pnpm tauri dev
```

## ⚠️ 未签名安装提示（首版）

- 当前首版默认未做代码签名/公证，优先打通发布链路。
- macOS 首次运行可能提示“无法验证开发者”，可在“系统设置 > 隐私与安全性”放行。
- Windows 可能出现 SmartScreen 提示，可选择“仍要运行”继续安装。

## 📄 许可证

本项目仅供学习和个人使用。请尊重音乐版权，仅下载您拥有合法使用权的内容。

## ⚠️ 免责声明

本工具仅用于技术研究和个人学习目的。用户需自行承担使用本工具的法律责任，开发者不对任何版权问题承担责任。
