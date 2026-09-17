<div align="center">

<img src="static/logo.svg" width="96" height="96" alt="ClipFort Logo" />

# ClipFort

### *Ultra-Lightweight, Cross-Platform Clipboard Manager & Encrypted Vault*

[![Release](https://img.shields.io/badge/release-v0.1.0-emerald.svg)](https://github.com)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%2010%2B-blue.svg)](https://github.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Built With](https://img.shields.io/badge/built%20with-Tauri%20v2%20%2B%20Rust%20%2B%20SvelteKit-orange.svg)](https://tauri.app)
[![Memory Footprint](https://img.shields.io/badge/RAM-~15--20%20MB-success.svg)](https://github.com)

<br/>

**ClipFort** is a lightning-fast, ultra-lightweight desktop clipboard manager and encrypted vault designed for both **Linux** and **Windows 10/11**. Engineered with Rust and Tauri v2, it consumes minimal system resources (~15-20 MB RAM, near-zero CPU idle) while providing a seamless Raycast/Spotlight-style floating overlay, real-time color/URL previews, and an **AES-256-GCM encrypted Secure Vault** for your sensitive credentials and files.

</div>

---

## ⚡ Why ClipFort?

Traditional clipboard managers either consume hundreds of megabytes of RAM (Electron-based) or store your sensitive copied passwords, API tokens, and credentials in plaintext, exposing them to any process running on your machine.

**ClipFort** solves both problems:
1. **Ultra-Lightweight Footprint:** Built on native platform webviews (WebKitGTK on Linux, WebView2 on Windows) and a compiled Rust core. It launches instantly and stays out of your system's way.
2. **Dual Clipboard & Encrypted Vault:** Fast, frictionless clipboard history for everyday workflows, combined with an isolated, hardware-bound or PIN-protected **Secure Vault** that keeps critical secrets encrypted at rest.
3. **100% Offline & Private:** Zero cloud dependencies, zero telemetry, and zero network calls. Your data stays strictly on your machine.

---

## ✨ Features

### 📋 1. Smart Clipboard History
- **Dual Content Capture:** Automatically monitors and captures both plain text/code snippets and copied screenshots/images.
- **Smart Bump-to-Top:** Frequently used items rise to the top of the history list when copied or selected.
- **Instant Number Shortcuts (`1` - `9`):** Copy any of the top 9 items instantly with a single keystroke.
- **Color & URL Previews:**
  - Detects `#HEX`, `rgb()`, and `hsl()` color codes and renders a live color badge directly on the card.
  - Automatically recognizes URLs and web links, providing a direct 🌐 **Open in Browser** shortcut button.
- **Categories & Instant Search:** Filter by `All`, `Text`, `Images`, or `Pinned` with sub-millisecond fuzzy search.
- **Export Clipboard History:** Backup or export your clipboard text history to Markdown (`.md`) or Plain Text (`.txt`) at any time.

### 🔐 2. Encrypted Secure Vault
- **Military-Grade Cryptography:** Protected by **AES-256-GCM** authenticated encryption with **PBKDF2-HMAC-SHA256** (100,000 iterations and a 32-byte cryptographically secure salt).
- **Flexible Security Modes:**
  - **Automatic Device Encryption (Default):** Zero prompts. Automatically unlocks on your machine using a unique machine-bound key derived from hardware identifiers.
  - **Master PIN Protection (Optional):** Require a custom PIN to unlock the vault each session.
- **Secure File & Document Storage:**
  - Store identity scans, PDFs, SSH keys, certificates, and documents (up to 20+ MB configurable limit).
  - **Two Storage Methods:** Create a lightweight **Shortcut** to the original path, or **Copy to Vault** for permanent encrypted retention.
  - Seamless native clipboard file pasting via GTK URI / Windows Shell protocol.
- **Full Vault Backup & Restore (`.vaultbak`):**
  - Export all passwords, notes, categories, and attached documents into a single encrypted backup archive to transfer securely across computers.
- **Categorized Boards & Groups:** Organize records into customizable tabs (e.g. *Personal Info*, *Websites & Accounts*, *Development & Keys*, *Notes & Documents*) and custom group containers.

### 🎨 3. Ergonomics & Customization
- **Modern Floating Overlay:** Frameless, compact Raycast/Spotlight design with a custom drag handle. Remembers window size and custom position.
- **Curated Theme Presets + Custom Palette:**
  - Modern Dark, Light Minimal, Midnight Blue, Emerald Forest, Cyber Violet, Warm Amber, or full custom RGB color picker.
- **Autostart on Boot:** Enable launch on system boot with one click (supports XDG autostart on Linux and Windows Registry).
- **Customizable Shortcuts & Behavior:** Configure the global hotkey (`Alt+Shift+Q` by default), window width/height, and whether the overlay closes after copying.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Description |
|---|---|
| `Alt + Shift + Q` | Toggle ClipFort overlay window (customizable in Settings) |
| `↑` / `↓` | Navigate through clipboard items |
| `Enter` | Copy selected item to clipboard, bump to top, and close overlay |
| `1` – `9` | Instantly copy items 1 through 9 with a single keystroke |
| `P` | Toggle Pin on selected item |
| `Delete` | Delete selected item (with confirmation) |
| `Alt + ↑` / `Alt + ↓` | Move selected item manually up / down in the list |
| `Esc` | Close open modal, or hide ClipFort window |

---

## 📥 Installation

### Linux (Debian / Ubuntu / Linux Mint)

Download the latest `.deb` package from releases and install:

```bash
# Install the Debian package
sudo dpkg -i clipfort_0.1.0_amd64.deb

# Resolve any missing system dependencies if needed
sudo apt-get install -f
```

Once installed, **ClipFort** is accessible from your application launcher or by running `clipfort` in any terminal.

### Windows 10 / 11

Download `clipfort_0.1.0_x64-setup.exe` or `clipfort.msi` from releases and run the installer. ClipFort automatically minimizes to the system tray and registers `Alt+Shift+Q` as the global hotkey.

---

## 🛠️ Building from Source

### Prerequisites

- **Node.js**: v18+ or v20+ (`node -v`)
- **Rust & Cargo**: v1.75+ (`cargo --version`)

#### Linux Dependencies:
```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

#### Windows Dependencies:
- Visual Studio C++ Build Tools (with Windows SDK)
- Microsoft Edge WebView2 (pre-installed on Windows 10/11)

### Development Mode

```bash
# 1. Clone the repository
git clone https://github.com/username/clipfort.git
cd clipfort

# 2. Install frontend dependencies
npm install

# 3. Launch in development mode with hot reload
npm run tauri dev
```

### Production Build

```bash
# Builds release binary and platform installers (.deb / .exe)
npm run tauri build
```

The compiled release packages will be located in:
- **Linux:** `src-tauri/target/release/bundle/deb/clipfort_0.1.0_amd64.deb`
- **Windows:** `src-tauri/target/release/bundle/nsis/clipfort_0.1.0_x64-setup.exe`

---

## 🔒 Security & Privacy Architecture

1. **Local-First & Zero-Cloud:**
   ClipFort never communicates with any external server. All configuration, clipboard history, and encrypted vault storage reside locally:
   - **Linux:** `~/.local/share/clipfort/`
   - **Windows:** `%APPDATA%\clipfort\`
2. **Cryptographic Standards:**
   - **Cipher:** `AES-256-GCM` (Galois/Counter Mode for authenticated encryption with associated data - AEAD).
   - **Key Derivation Function (KDF):** `PBKDF2-HMAC-SHA256` with 100,000 rounds and unique 32-byte salt.
   - **File System Permissions:** On Linux, vault directories are secured with strict `0700` POSIX permissions, preventing unauthorized access from other local users.
3. **Seamless Migration:**
   ClipFort includes built-in backward compatibility that automatically discovers and upgrades legacy databases without data loss.

---

## 📄 License

This project is licensed under the [MIT License](LICENSE). Feel free to inspect, fork, and contribute!
