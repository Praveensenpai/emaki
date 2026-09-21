<div align="center">

# 絵巻 (Emaki)

### *Ultra-Lightweight, Zero-Overhead WhatsApp Instagram Reel Relay Daemon*

[![Rust](https://img.shields.io/badge/Rust-1.94%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Memory: ~15MB](https://img.shields.io/badge/Memory-~15MB%20RAM-brightgreen.svg?style=flat-square)](https://github.com/Praveensenpai)
[![E2EE: Native Signal](https://img.shields.io/badge/E2EE-Signal%20Protocol-purple.svg?style=flat-square)](https://github.com/oxidezap/whatsapp-rust)

<p align="center">
  <b>Never install Instagram. Never doom-scroll. Watch shared reels directly inside your WhatsApp group.</b>
</p>

</div>

---

## 📖 Overview

**絵巻 (Emaki)** is a standalone WhatsApp companion daemon written in pure **Rust**. When friends or group members share Instagram Reel URLs into your group chat, Emaki intercepts the link, fetches the video asynchronously via `yt-dlp` (with automatic retry resilience and a 5GB LRU local cache), and re-uploads the native `.mp4` video directly to the chat with end-to-end encryption.

No Instagram account required. No browsers, Puppeteer, or heavy Node runtimes.

---

## 🪄 One-Liner Magic

Install `emaki` directly to `~/.local/bin/emaki` with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/Praveensenpai/emaki/main/install.sh | bash
```

---

## ⚡ Why Rust?

| Metric | Traditional Node.js (Baileys) | Python (`neonize`) | **絵巻 (Emaki - Rust)** |
| :--- | :--- | :--- | :--- |
| **Idle Memory** | ~120 MB | ~140 MB | **~10 – 20 MB** ⚡ |
| **Startup Time** | ~2.5s | ~1.8s | **< 5ms** |
| **Runtime Dependencies** | Node, npm, Chromium/Puppeteer | Python, pip, GCC | **Single Native Binary** |
| **Disk Overhead** | Hundreds of MBs in `node_modules` | Virtualenv packages | **Zero** |

---

## 📐 Architecture & Workflow

```text
┌─────────────────────────────────────────────────────────────┐
│                   絵巻 (Emaki) Architecture                 │
│                                                             │
│   WhatsApp Group        whatsapp-rust (Tokio Runtime)       │
│   [Reel Link]   ──────> on_message hook                     │
│                                │ (~15 MB RAM)               │
│                                ▼                            │
│                        ReelExtractor                        │
│                     Regex & Canonical ID                    │
│                                │                            │
│                                ▼                            │
│                     FIFO Queue (tokio::mpsc)                │
│                     (3s anti-ban cooldown)                  │
│                                │                            │
│                                ▼                            │
│                     5GB LRU Video Cache                     │
│                  (Physical file validation)                 │
│                  ├── Hit ──> Direct Upload                  │
│                  └── Miss ─> yt-dlp (3x Retry)              │
│                                │                            │
│                                ▼                            │
│                     WhatsApp E2EE Media Upload              │
│                     Rich Caption with Blockquotes           │
└─────────────────────────────────────────────────────────────┘
```

---

## ✨ Features

- 📱 **Personal Number Friendly**: Pairs seamlessly as a standard **Linked Device** via QR code rendered directly in your terminal.
- 🚦 **Anti-Ban FIFO Queue**: Incoming reels are queued and processed sequentially with a mandatory **3-second cooldown gap** to prevent Meta spam detection. Zero burst reactions.
- 🔁 **3-Attempt Resilient Download**: Automatically retries failed reel downloads up to 3 times with progressive backoff.
- 🗄️ **5GB Smart LRU Cache**: Stores downloaded videos and metadata locally. If a reel is re-shared, it uploads instantly without hitting Instagram. Automatically prunes the oldest files when exceeding 5GB.
- 🎨 **Rich Formatting**: Formats captions with WhatsApp blockquotes (`> ...`), author attribution (`@creator`), and clean links.
- 🎯 **Group Whitelist**: Restrict the bot to only listen to specific group chats to prevent unwanted triggers.
- 🔒 **End-to-End Encrypted**: Built on `whatsapp-rust`, implementing WhatsApp Web's Signal Protocol and Noise handshake.

---

## 🚀 Quick Start

### 1. Prerequisites
Ensure you have `yt-dlp` and `ffmpeg` installed on your system:
```bash
# Arch Linux
sudo pacman -S yt-dlp ffmpeg

# Ubuntu / Debian
sudo apt update && sudo apt install yt-dlp ffmpeg
```

### 2. Run
```bash
emaki
```
Or from source:
```bash
git clone https://github.com/Praveensenpai/emaki.git
cd emaki
cargo run --release
```

### 3. Pair With WhatsApp
1. On startup, Emaki will display a high-resolution QR code directly in your terminal.
2. Open WhatsApp on your phone:
   - **Settings** (or three dots) > **Linked Devices** > **Link a Device**.
3. Scan the terminal QR code.
4. Your session is saved locally in `emaki.db` (SQLite). Future runs will auto-connect without re-scanning.

---

## ⚙️ Configuration (`emaki.toml`)

Create an optional `emaki.toml` file in the working directory:

```toml
# Path to the SQLite session database
session_db = "emaki.db"

# Local cache directory for instant re-sharing of viral reels
cache_dir = "cache"

# Maximum cache size in gigabytes before auto-pruning oldest files (LRU)
max_cache_size_gb = 5

# WhatsApp Group Whitelist
# Leave empty ([]) to respond in any group where the bot is present.
# To restrict to specific groups, specify their JIDs:
whitelist_groups = [
  # "12036302XXXXXXXXXX@g.us"
]

# Temporary video buffering path
temp_dir = "/tmp"

# Maximum download file size in MB
max_file_size_mb = 50

# Custom caption prepended to uploaded reels
caption_prefix = "🎬 Reel via 絵巻"
```

---

## 🛡️ Best Practices & Anti-Ban Tips

- **Whitelist Only**: Keep `whitelist_groups` configured to only your private friends' group.
- **Pacing is Built-in**: The bot enforces a 3-second delay between video deliveries and suppresses burst emoji reactions.
- **Natural Usage**: Emaki responds only when a group member shares a link—it sends zero unsolicited messages.

---

## 📄 License

Distributed under the **MIT License**.
