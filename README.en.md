<div align="center">

<img src="src-tauri/icons/icon.png" width="96" alt="ClipBox Logo">

# ClipBox

**A lightweight, local-first clipboard toolbox**

Capture copy history automatically — search, pin, and paste in one keystroke. No cloud sync, low footprint, your data stays on your machine.

<br>

[![Tauri 2](https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white)](https://tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Svelte 5](https://img.shields.io/badge/Svelte-5-FF3E00?logo=svelte&logoColor=white)](https://svelte.dev/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux-lightgrey)](#-platform-notes)

**[中文 README](./README.md)** · [Quick Start](#-quick-start) · [Features](#-features) · [Development](#-development)

</div>

---

## Why ClipBox?

| | ClipBox | Typical cloud clipboard |
|---|---|---|
| Storage | Local SQLite, no upload | Account & sync required |
| Footprint | Tauri native shell, lightweight | Often heavier |
| Privacy | App allow/deny lists, pause capture | Depends on vendor |
| UX | Tray + global hotkey, keyboard-first | Varies |

> **ClipBox** = *Clip* + *Box* — a box for everything you copy. Chinese nickname: **剪匣** (*jiǎn xiá*, "clipboard casket").

---

## Features

### Core

- **Clipboard history** — Text, HTML, images, and file paths; optional deduplication
- **Full-text search** — SQLite FTS5
- **Pinned items** — Excluded from auto-cleanup
- **Global hotkey** — Default `Ctrl+Shift+V` to show/hide the panel
- **Paste as plain text** — Strip HTML formatting in one action
- **Snippets** — Reusable templates with drag-to-reorder
- **System tray** — Runs in background; left-click toggles panel

### Advanced

- **Smart search** — Natural language queries, e.g. `yesterday chrome link`
- **Tag filters** — `#code` / `#url` / `#img`
- **Grouping** — By time (today, yesterday, this week) or source app
- **Minimal mode** — Win+V-style card layout
- **Save images** — Context menu or `↓` button to a chosen folder
- **App filter** — Blacklist / whitelist for capture sources
- **Backup** — Export / import; custom storage path and retention
- **Theming & i18n** — System / light / dark; Chinese & English UI

### Philosophy

```
Local-first · Optional features · Low overhead · Keyboard-friendly
```

Advanced capabilities can be toggled individually in Settings.

---

## Quick Start

### Install (Windows)

1. Download the latest release from [Releases](https://github.com/Hyperion-WB/-ClipBox/releases)
2. Recommended: `ClipBox_x64-setup.exe` (NSIS installer)
3. ClipBox lives in the system tray after install

### Basic usage

1. Copy as usual — ClipBox records in the background
2. Press **`Ctrl+Shift+V`** or click the tray icon
3. Use **`↑` `↓`** to select, **`Enter`** to paste, **`Esc`** to close
4. Open the **Snippets** tab for saved templates
5. Adjust hotkey, theme, language, and retention in **Settings**

---

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| `Ctrl+Shift+V` | Show / hide panel (configurable) |
| `↑` / `↓` | Move selection |
| `1` – `9` | Paste item by index |
| `Enter` | Paste selection |
| `Ctrl+Shift+Enter` | Paste as plain text (HTML items) |
| `Ctrl+F` | Focus search |
| `Ctrl+A` | Enter multi-select mode |
| `Delete` | Batch delete (multi-select) |
| `Esc` | Close menu / exit multi-select / hide panel |
| `Alt+1` – `Alt+4` | Filter: All / Text / Image / File |

---

## Tech stack

| Layer | Technology |
|-------|------------|
| Shell | [Tauri 2](https://tauri.app/) |
| Backend | Rust — clipboard monitor, SQLite, paste simulation, notifications |
| Frontend | Svelte 5 + SvelteKit |
| Storage | SQLite + FTS5 |
| Data dir | Windows: `%LOCALAPPDATA%/clipbox` · Linux: `~/.local/share/clipbox` |

---

## Development

### Requirements

- **Node.js** 18+
- **Rust** 1.77+
- **Windows**: WebView2 (preinstalled on Win 10+)
- **Linux**: `webkit2gtk`, `libayatana-appindicator`, etc. — see [Tauri prerequisites](https://tauri.app/start/prerequisites/)

### Run locally

```bash
git clone https://github.com/Hyperion-WB/-ClipBox.git
cd ClipBox
npm install
npm run tauri dev
```

### Build installers

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/`

- Windows: `nsis/ClipBox_*_x64-setup.exe`, `msi/ClipBox_*_x64_en-US.msi`

### Useful scripts

```bash
npm run check          # Svelte / TypeScript check
npm run generate-icon  # Regenerate app icons
```

### Project layout

```
ClipBox/
├── src/                    # Svelte frontend
│   ├── lib/
│   │   ├── components/
│   │   ├── locales/        # zh.ts / en.ts
│   │   └── i18n.svelte.ts
│   └── routes/
├── src-tauri/              # Rust backend
│   └── src/
│       ├── db/
│       ├── clipboard/
│       └── search.rs
├── assets/
└── scripts/
```

---

## Platform notes

### Windows

Works out of the box. If simulated `Ctrl+V` paste fails, disable **Simulate Ctrl+V on select** under **Settings → General** and paste manually.

> ClipBox **cannot replace** the system **Win+V** clipboard history natively. Use the ClipBox hotkey, or remap Win+V via [PowerToys](https://github.com/microsoft/PowerToys).

### Linux

| Environment | Notes |
|-------------|-------|
| **X11** | Full clipboard monitoring |
| **Wayland** | Some apps block clipboard reads without focus; opening the panel usually fixes this |
| **Deepin** | Disable system `dde-clipboard` if both run, to avoid conflicts |

---

## Privacy

- All data stays **on your machine** — no cloud sync
- **Pause capture**, **app filters**, export/import backup
- Open source — issues and PRs welcome

---

## Acknowledgements

Inspired by (not forked from):

- [dde-clipboard](https://github.com/linuxdeepin/dde-clipboard)
- [CopyQ](https://github.com/hluk/CopyQ)
- [ClipMan](https://github.com/RustyPiano/ClipMan)

---

## Contributing

1. Fork the repo
2. Create a branch: `git checkout -b feature/amazing-feature`
3. Commit: `git commit -m 'Add amazing feature'`
4. Push: `git push origin feature/amazing-feature`
5. Open a Pull Request

---

## License

[MIT License](LICENSE)
