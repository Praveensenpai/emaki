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

**絵巻 (Emaki)** is a standalone WhatsApp companion daemon written in pure **Rust**. When friends or group members share Instagram Reel URLs into your group chat, Emaki intercepts the link, fetches the video asynchronously via `yt-dlp` (with 3-attempt retry resilience and a 5GB LRU local cache), and re-uploads the native `.mp4` video directly to the chat with end-to-end encryption.

No Instagram account required. No browsers, Puppeteer, or heavy Node runtimes.

---

## 🪄 One-Liner Magic

Install the pre-compiled `emaki` binary directly to `~/.local/bin/emaki` with a single command:

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
- 🛡️ **Daemon Session Guard**: Separates interactive pairing (`emaki login`) from headless execution (`emaki daemon`), ensuring background daemons never hang without credentials.
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

### 2. Install Emaki
```bash
curl -fsSL https://raw.githubusercontent.com/Praveensenpai/emaki/main/install.sh | bash
```

### 3. Pair With WhatsApp
Run the interactive login command in your terminal:
```bash
emaki login
```
1. A high-resolution QR code will render directly in your terminal.
2. Open WhatsApp on your phone:
   - **Settings** (or three dots) > **Linked Devices** > **Link a Device**.
3. Scan the terminal QR code.
4. Your cryptographic session keys are saved locally in `emaki.db` (SQLite).

### 4. Verify & Run Daemon

Check authentication & daemon status from any directory:
```bash
emaki status
```

Start the daemon in the background (non-blocking):
```bash
emaki daemon
```

Follow live logs or stop the background daemon:
```bash
emaki logs   # Follow live logs
emaki stop   # Stop background daemon
```

---

## 🐧 24/7 Auto-Start on Reboot (Systemd)

Install and enable the 24/7 background service with a single command:
```bash
emaki service install
```
This automatically configures a rootless systemd user unit (`~/.config/systemd/user/emaki.service`) with auto-restart on network drop and persistent boot execution (`loginctl enable-linger`).

Service lifecycle commands:
```bash
emaki service status     # Check background service status
emaki service logs       # View live systemd service logs
emaki service restart    # Restart background daemon
emaki service stop       # Stop service
emaki service uninstall  # Remove systemd service
```

---

### Option B: System Service (Root / Multi-User)

Create `/etc/systemd/system/emaki.service`:

```ini
[Unit]
Description=絵巻 (Emaki) WhatsApp Reel Daemon
After=network.target

[Service]
Type=simple
User=<your-username>
WorkingDirectory=/home/<your-username>
ExecStart=/home/<your-username>/.local/bin/emaki daemon
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable --now emaki
journalctl -u emaki -f
```

---

## ⚙️ Configuration (Zero-Manual CLI Management)

Emaki requires **zero manual file creation**. On first run, it automatically initializes sensible defaults inside `~/.emaki/emaki.toml`.

Manage all settings directly from your terminal without opening any files:

### 1. View Active Configuration
```bash
emaki config
```

### 2. Modify Settings via CLI
```bash
# Set download file size limit in MB (default: 500)
emaki config set max_file_size_mb 250

# Set maximum LRU cache limit in GB (default: 5)
emaki config set max_cache_size_gb 10

# Set custom caption prefix
emaki config set caption_prefix "🎬 Shared Reel"
```

### 3. Restrict to Specific Groups (Whitelist)
By default, the bot responds in any group where it is a member. You can restrict it via CLI:
```bash
# Add a group JID to whitelist
emaki whitelist add 120363024819283746@g.us

# List all whitelisted groups
emaki whitelist list

# Remove a group from whitelist
emaki whitelist remove 120363024819283746@g.us
```

---

## 🛡️ Best Practices & Anti-Ban Tips

- **Whitelist Only**: Keep `whitelist_groups` configured to only your private friends' group.
- **Pacing is Built-in**: The bot enforces a 3-second delay between video deliveries and suppresses burst emoji reactions.
- **Transferring to VPS**: Pair locally using `emaki login`, then copy `emaki.db` to your remote server (`scp emaki.db user@remote:/path/to/`). The remote daemon will boot immediately without needing a QR scan!

---

## 📄 License

Distributed under the **MIT License**.
