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
- **Fully Customizable Shortcuts:** Customize all shortcuts (global launcher, copy, delete, pin, reorder, search, mode switch, settings, clear, export) with live key recording, conflict detection, and reset options in **Settings → Shortcuts**.
- **Themes:** Dark, Light, Midnight Navy, Emerald Forest, Cyber Violet, Warm Amber, Nord Frost, Pure Black (OLED), or custom colors.
- **Window Positioning:** Stays anchored at the bottom-right corner by default, or remembers where you drag it.
- **Autostart:** Optional launch on system startup.

---

## Keyboard Shortcuts

All shortcuts are fully customizable from **Settings → Shortcuts**. Below are the default keybindings:

| Default Shortcut | Action | Scope | Customizable |
|---|---|---|---|
| `Alt + Shift + Q` | Toggle ClipFort window | Global (System-wide) | Yes |
| `↑` / `↓` | Navigate clipboard items | In-app | Navigation |
| `Enter` | Copy selected item and close window | In-app | Yes |
| `1` – `9` | Instantly copy items 1 through 9 | In-app | Toggleable |
| `P` | Toggle pin on selected item | In-app | Yes |
| `Delete` | Delete selected item | In-app | Yes |
| `Alt + ↑` / `Alt + ↓` | Move selected item up or down | In-app | Yes |
| `Ctrl + F` | Focus search bar | In-app | Yes |
| `Ctrl + Delete` | Clear unpinned clipboard history | In-app | Yes |
| `Escape` | Close modal, or hide ClipFort window | In-app | Yes |
| `Ctrl + Tab` | Switch between Clipboard & Vault | In-app | Yes |
| `Ctrl + ,` | Open Settings dialog | In-app | Yes |
| `Ctrl + E` | Open Export Clipboard History | In-app | Yes |

---

## Installation

Pre-built releases for Linux and Windows are available on the [Releases](https://github.com/username/clipfort/releases) page.

### 🐧 Linux (Ubuntu / Debian / Mint / Pop!_OS)

#### Option 1: Install via `.deb` Package (Recommended)
1. Download `ClipFort_0.1.0_amd64.deb` from [Releases](https://github.com/username/clipfort/releases).
2. Install via terminal or double-click in your file manager:
   ```bash
   sudo dpkg -i ClipFort_0.1.0_amd64.deb
   sudo apt-get install -f   # resolves any dependencies automatically if needed
   ```
3. Launch **ClipFort** from your application menu or run `clipfort` in a terminal.
4. **Hotkeys & Tray:** ClipFort starts minimized in the system tray and opens with `Alt + Shift + Q`.

#### Option 2: Standalone Binary (Arch / Fedora / Other Distros)
If you prefer not to use `.deb`:
1. Extract the release package.
2. Copy the binary to `/usr/local/bin/`:
   ```bash
   sudo cp clipfort /usr/local/bin/
   sudo chmod +x /usr/local/bin/clipfort
   ```

---

### 🪟 Windows (Windows 10 / 11)

#### Option 1: NSIS Setup Installer (`.exe`)
1. Download `clipfort_0.1.0_x64-setup.exe` from [Releases](https://github.com/username/clipfort/releases).
2. Run the installer and follow the setup wizard.
3. ClipFort launches in your **System Tray** (near the taskbar clock).
4. Press `Alt + Shift + Q` anytime to summon ClipFort.
5. **Autostart:** To run on boot, open **Settings (`Ctrl + ,`) → General** and toggle on **"Launch on startup"**.

#### Option 2: Portable Standalone Executable
1. Download the portable zip archive from [Releases](https://github.com/username/clipfort/releases).
2. Extract the folder anywhere (e.g. `C:\Program Files\ClipFort\`).
3. Run `clipfort.exe`.

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

To compile ClipFort yourself from source:

### Prerequisites (All Platforms)
- **Node.js:** v18, v20, or v22 ([Download Node.js](https://nodejs.org/))
- **Rust:** Latest stable toolchain ([Install Rust via rustup](https://rustup.rs/))

### 🐧 Linux Build Instructions

1. **Install required build dependencies:**
   ```bash
   sudo apt-get update
   sudo apt-get install -y \
     libwebkit2gtk-4.1-dev \
     libgtk-3-dev \
     libayatana-appindicator3-dev \
     librsvg2-dev \
     patchelf \
     build-essential \
     curl
   ```

2. **Clone repository and install frontend dependencies:**
   ```bash
   git clone https://github.com/username/clipfort.git
   cd clipfort
   npm install
   ```

3. **Run in development mode (live reload):**
   ```bash
   npm run tauri dev
   ```

4. **Build release `.deb` package:**
   ```bash
   npm run tauri build -- --bundles deb
   ```
   Output: `src-tauri/target/release/bundle/deb/ClipFort_0.1.0_amd64.deb`

---

### 🪟 Windows Build Instructions

1. **Install prerequisites:**
   - [Visual Studio 2022 Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (check **"Desktop development with C++"**).
   - [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (pre-installed on Windows 10/11).

2. **Clone repository and install dependencies (PowerShell):**
   ```powershell
   git clone https://github.com/username/clipfort.git
   cd clipfort
   npm install
   ```

3. **Run in development mode:**
   ```powershell
   npm run tauri dev
   ```

4. **Build release installer (`.exe`):**
   ```powershell
   npm run tauri build -- --bundles nsis
   ```
   Output: `src-tauri\target\release\bundle\nsis\clipfort_0.1.0_x64-setup.exe`

---

## Data Storage & Security

- **Linux Storage:** `~/.local/share/clipfort/`
- **Windows Storage:** `%APPDATA%\clipfort\`
- **Encryption:** AES-256-GCM authenticated encryption.
- **Vault File Permissions:** Set to `0700` (read/write only by the current user) on Unix systems.
- **Network:** Zero network requests. Operates completely offline.

---

## License

MIT License.
