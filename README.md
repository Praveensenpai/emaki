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

**絵巻 (Emaki)** is a standalone WhatsApp companion daemon written in pure **Rust**. When friends or group members share Instagram Reel URLs into your group chat, Emaki intercepts the link, fetches the video asynchronously via `yt-dlp`, and re-uploads the native `.mp4` video directly to the chat with end-to-end encryption.

No Instagram account required. No browsers, Puppeteer, or heavy Node runtimes.

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
│                        ReelDownloader                       │
│                     yt-dlp (async spawn)                    │
│                                │                            │
│                                ▼                            │
│                     WhatsApp E2EE Media Upload              │
│                     Stream & Send to Group Chat             │
│                                │                            │
│                                ▼                            │
│                     Instant Tempfile Cleanup                │
└─────────────────────────────────────────────────────────────┘
```

---

## ✨ Features

- 📱 **Personal Number Friendly**: Pairs seamlessly as a standard **Linked Device** via QR code rendered directly in your terminal.
- ⚡ **Instant Processing**: Reacts with `⏳` when downloading starts, replaces with `✅` upon delivery, or `❌` if failed.
- 🛡️ **Zero Disk Residue**: Temporary videos are deleted immediately after the encrypted upload completes.
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

### 2. Build & Run
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
- **Never Spam**: Do not use the bot for broadcast lists or unsolicited personal chats.
- **Natural Usage**: Emaki responds only when a group member shares a link—it sends zero unsolicited messages.

---

## 📄 License

Distributed under the **MIT License**.
