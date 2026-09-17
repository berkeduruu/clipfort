<div align="center">

<img src="static/logo.svg" width="96" height="96" alt="ClipFort Logo" />

# ClipFort

### Clipboard Manager & Encrypted Vault for Linux and Windows

[![Release](https://img.shields.io/badge/release-v0.1.0-emerald.svg)](https://github.com)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-blue.svg)](https://github.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Built With](https://img.shields.io/badge/built%20with-Tauri%20v2%20%2B%20Rust%20%2B%20Svelte-orange.svg)](https://tauri.app)

<br/>

**ClipFort** is a desktop clipboard manager and encrypted vault for Linux and Windows. It runs in the background, captures copied text and images, and provides an encrypted vault for storing passwords, notes, and documents.

</div>

---

## Overview

ClipFort includes two main features:

1. **Clipboard History:** Automatically records copied text, code snippets, and images. Lets you search, reorder, pin, and paste previous clips using keyboard shortcuts.
2. **Encrypted Vault:** A local encrypted store for sensitive notes, passwords, and files, protected with AES-256-GCM.
3. **Offline:** Works entirely locally. No network access, cloud sync, or telemetry.

---

## Features

### 📋 Clipboard History
- **Text & Image Monitoring:** Automatically captures copied text and image screenshots.
- **Search & Filters:** Instant search with filters for `All`, `Text`, `Images`, and `Pinned`.
- **Keyboard Navigation:** Arrow keys to navigate, `Enter` to copy and paste, `1`–`9` keys to directly copy the top 9 items.
- **Color & URL Detection:**
  - Shows a live color badge for `#HEX`, `rgb()`, and `hsl()` color codes.
  - Detects URLs and provides an "Open in Browser" button.
- **Reordering & Pinning:** Pin important clips to the top, or reorder them manually with `Alt + ↑/↓`.
- **Export History:** Export text history as Markdown (`.md`) or plain text (`.txt`).

### 🔐 Encrypted Vault
- **AES-256-GCM Encryption:** Data is encrypted locally using AES-256-GCM with key derivation via PBKDF2-HMAC-SHA256 (100,000 iterations).
- **Unlock Modes:**
  - **Device-Bound Key (Default):** Automatically decrypts using a key tied to your machine.
  - **Master PIN (Optional):** Requires a user PIN to unlock each session.
- **File Storage:** Attach files, PDFs, or keys to vault items as shortcuts or copied directly into vault storage.
- **Backup & Restore:** Export and import the vault as a single encrypted `.vaultbak` file.
- **Tabs & Groups:** Organize records into custom tabs and groups.

### ⚙️ Settings & Customization
- **Global Hotkey:** `Alt + Shift + Q` by default (customizable).
- **Themes:** Dark, Light, Midnight Blue, Forest, Cyber Violet, Amber, or custom colors.
- **Window Positioning:** Stays anchored at the bottom-right corner by default, or remembers where you drag it.
- **Autostart:** Optional launch on system startup.

---

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Alt + Shift + Q` | Toggle ClipFort window (customizable in Settings) |
| `↑` / `↓` | Navigate clipboard items |
| `Enter` | Copy selected item to clipboard and close window |
| `1` – `9` | Instantly copy items 1 through 9 |
| `P` | Toggle pin on selected item |
| `Delete` | Delete selected item |
| `Alt + ↑` / `Alt + ↓` | Move selected item up or down in the list |
| `Esc` | Close modal, or hide ClipFort window |

---

## Installation

### Linux (Debian / Ubuntu)

Download the `.deb` package from releases and install:

```bash
sudo dpkg -i ClipFort_0.1.0_amd64.deb
sudo apt-get install -f   # if any dependencies are needed
```

Launch from your application menu or run `clipfort` in a terminal.

### Windows 10 / 11

Download and run `clipfort_0.1.0_x64-setup.exe` from releases. ClipFort runs in the system tray and registers `Alt+Shift+Q`.

---

## System Resources & Performance

Measured on Linux (Ubuntu 24.04, X11, WebKitGTK):

- **Package Size:** ~3.1 MB (`.deb`)
- **Binary Size:** ~7.4 MB
- **Idle CPU:** `< 0.1%` (no polling loops or background work when idle)
- **Memory (RSS):** ~170 MB in process monitors (`btop`, `htop`, `top`). This includes shared WebKitGTK and system libraries.
- **Private Heap:** ~30 MB (actual memory allocated by ClipFort itself).

---

## Building from Source

### Prerequisites
- Node.js v18+ or v20+
- Rust & Cargo v1.75+

#### Linux Build Dependencies:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

#### Windows Build Dependencies:
- Visual Studio C++ Build Tools
- Microsoft Edge WebView2

### Development
```bash
git clone https://github.com/username/clipfort.git
cd clipfort
npm install
npm run tauri dev
```

### Build Release
```bash
npm run tauri build
```

The output packages are in:
- **Linux:** `src-tauri/target/release/bundle/deb/ClipFort_0.1.0_amd64.deb`
- **Windows:** `src-tauri/target/release/bundle/nsis/clipfort_0.1.0_x64-setup.exe`

---

## Data Storage & Security

- **Linux Storage:** `~/.local/share/clipfort/`
- **Windows Storage:** `%APPDATA%\clipfort\`
- **Encryption:** AES-256-GCM authenticated encryption.
- **Vault File Permissions:** Set to `0700` (read/write only by the current user) on Unix systems.
- **Network:** Zero network requests. Operates completely offline.

---

## License

MIT License. See [LICENSE](LICENSE) for details.
