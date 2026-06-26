<div align="center">

<img src="src-tauri/icons/icon.png" width="96" alt="ClipBox Logo">

# ClipBox · 剪匣

**轻量、本地优先的剪贴板工具箱**

自动记录复制历史，一键搜索、置顶与粘贴 —— 无云同步，低占用，数据留在本机。

<br>

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey)](#-平台说明)

**[English README](./README.en.md)** · [快速开始](#-快速开始) · [功能一览](#-功能一览) · [开发指南](#-开发指南)

</div>

---

## 为什么选择 ClipBox？

| | ClipBox | 典型云剪贴板 |
|---|---|---|
| 数据存储 | 本地 SQLite，不上传 | 需账号与同步 |
| 资源占用 | Tauri 原生壳，轻量后台 | 往往较重 |
| 隐私控制 | 应用黑白名单、可暂停记录 | 依赖服务商策略 |
| 开箱体验 | 托盘驻留 + 全局快捷键 | 各产品差异大 |

> **ClipBox** = *Clip*（剪贴）+ *Box*（收纳盒）。中文昵称 **剪匣** —— 把每一次复制，都收进自己的匣子里。

---

## 功能一览

### 核心能力

- **剪贴板历史** — 自动记录文本、HTML、图片与文件路径，支持去重
- **全文搜索** — SQLite FTS5 快速检索历史内容
- **置顶条目** — 常用内容置顶，不参与自动清理
- **全局快捷键** — 默认 `Ctrl+Shift+V` 呼出 / 隐藏面板
- **纯文本粘贴** — HTML 内容一键去除格式后粘贴
- **固定片段** — 保存邮箱、签名等常用模板，支持拖拽排序
- **系统托盘** — 后台驻留，左键切换面板，右键快捷菜单

### 进阶功能

- **智能搜索** — 自然语言查询，如「昨天 chrome 链接」
- **标签过滤** — `#code` / `#url` / `#img` 快速筛选
- **时间 / 来源分组** — 按今天、昨天、本周折叠，或按应用分组
- **极简模式** — 类 Win+V 的卡片式布局，聚焦内容与常用文本
- **图片保存** — 右键或 `↓` 按钮，保存图片到指定文件夹
- **应用过滤** — 黑名单 / 白名单控制记录来源
- **数据管理** — 导出 / 导入备份，自定义存储路径与保留策略
- **深色主题** — 跟随系统 / 浅色 / 深色，中英双语界面

### 设计原则

```
本地优先 · 功能可选 · 低占用 · 键盘友好
```

所有增强能力（分组、通知、智能搜索等）均可在设置中开关，按需启用。

---

## 快速开始

### 下载安装（Windows）

1. 前往 [Releases](https://github.com/Hyperion-WB/releases) 下载最新安装包
2. 推荐：`ClipBox_x64-setup.exe`（NSIS 安装程序）
3. 安装完成后，应用驻留系统托盘

### 基本使用

1. 正常复制内容，ClipBox 会在后台自动记录
2. 按 **`Ctrl+Shift+V`** 或点击托盘图标打开历史面板
3. 用 **`↑` `↓`** 选择条目，**`Enter`** 粘贴，**`Esc`** 关闭
4. 切换到 **片段** Tab 管理固定文本模板
5. 在 **设置** 中调整快捷键、主题、语言、保留策略等

---

## 快捷键

| 按键 | 作用 |
|------|------|
| `Ctrl+Shift+V` | 呼出 / 隐藏面板（可在设置中修改） |
| `↑` / `↓` | 上下选择条目 |
| `1` – `9` | 快速粘贴对应序号条目 |
| `Enter` | 粘贴选中条目 |
| `Ctrl+Shift+Enter` | 粘贴为纯文本（HTML 条目） |
| `Ctrl+F` | 聚焦搜索框 |
| `Ctrl+A` | 进入多选模式 |
| `Delete` | 多选模式下批量删除 |
| `Esc` | 关闭菜单 / 退出多选 / 隐藏面板 |
| `Alt+1` – `Alt+4` | 切换分类：全部 / 文本 / 图片 / 文件 |

---

## 技术栈

| 层级 | 技术 |
|------|------|
| 桌面壳 | [Tauri 2](https://tauri.app/) |
| 后端 | Rust — 剪贴板监听、SQLite、粘贴模拟、系统通知 |
| 前端 | Svelte 5 + SvelteKit |
| 存储 | SQLite + FTS5 |
| 数据目录 | Windows: `%LOCALAPPDATA%/clipbox` · Linux: `~/.local/share/clipbox` |

---

## 开发指南

### 环境要求

- **Node.js** 18+
- **Rust** 1.77+
- **Windows**：WebView2（Win 10+ 通常已预装）
- **Linux**：`webkit2gtk`、`libayatana-appindicator` 等 [Tauri 依赖](https://tauri.app/start/prerequisites/)

### 本地运行

```bash
git clone https://github.com/Hyperion-WB.git
cd ClipBox
npm install
npm run tauri dev
```

### 构建安装包

```bash
npm run tauri build
```

产物位于 `src-tauri/target/release/bundle/`：

- Windows：`nsis/ClipBox_*_x64-setup.exe`、`msi/ClipBox_*_x64_en-US.msi`

### 常用命令

```bash
npm run check          # Svelte / TypeScript 类型检查
npm run generate-icon  # 重新生成应用图标
```

### 项目结构

```
ClipBox/
├── src/                    # Svelte 前端
│   ├── lib/
│   │   ├── components/     # UI 组件
│   │   ├── locales/        # 中英文文案 (zh.ts / en.ts)
│   │   └── i18n.svelte.ts  # 国际化
│   └── routes/
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── db/             # SQLite 与设置
│       ├── clipboard/      # 剪贴板监听
│       └── search.rs       # FTS 与自然语言搜索
├── assets/                 # 图标源文件
└── scripts/                # 构建脚本
```

---

## 平台说明

### Windows

开箱即用。若「模拟 Ctrl+V」粘贴无效，可在 **设置 → 通用** 中关闭该选项，改为仅复制到剪贴板后手动 `Ctrl+V`。

> 无法原生替换系统 **Win+V** 剪贴板历史（Windows 限制）。建议使用 ClipBox 快捷键，或通过 [PowerToys](https://github.com/microsoft/PowerToys) 将 Win+V 重映射到 `Ctrl+Shift+V`。

### Linux

| 环境 | 说明 |
|------|------|
| **X11** | 完整支持剪贴板监听 |
| **Wayland** | 部分应用在无窗口焦点时无法读取剪贴板（Wayland 安全限制）；呼出面板获得焦点后通常正常 |
| **Deepin** | 若已启用系统 `dde-clipboard`，建议关闭以避免冲突 |

---

## 隐私与安全

- 所有剪贴板数据存储在**本机**，无网络同步
- 支持**暂停记录**、**应用黑白名单**过滤敏感来源
- 支持**导出 / 导入**备份，数据完全由你掌控
- 开源可审计，欢迎提交 Issue 与 PR

---

## 设计参考

本项目借鉴了以下开源项目的思路（非 fork）：

- [dde-clipboard](https://github.com/linuxdeepin/dde-clipboard) — 面板式历史与托盘集成
- [CopyQ](https://github.com/hluk/CopyQ) — 去重与纯文本粘贴
- [ClipMan](https://github.com/RustyPiano/ClipMan) — Tauri + SQLite 架构

---

## 参与贡献

欢迎 Star、Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 发起 Pull Request

---

## 许可证

本项目基于 [MIT License](LICENSE) 开源。
