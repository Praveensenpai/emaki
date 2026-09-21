#!/bin/bash
set -euo pipefail
IFS=$'\n\t'

CYAN='\033[0;36m'
GREEN='\033[0;32m'
PURPLE='\033[0;35m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${PURPLE}🎬  Installing 絵巻 (Emaki) — WhatsApp Instagram Reel Relay Daemon...${NC}\n"

BIN_DIR="$HOME/.local/bin"
mkdir -p "$BIN_DIR"

echo -e "${BLUE}🔍 Checking prerequisites...${NC}"

if command -v yt-dlp >/dev/null 2>&1; then
    echo -e "  ${GREEN}✔ yt-dlp is installed${NC}"
else
    echo -e "  ${YELLOW}⚠️  yt-dlp is not installed (required for downloading reels).${NC}"
    echo -e "     Install via: sudo pacman -S yt-dlp  or  sudo apt install yt-dlp"
fi

if command -v ffmpeg >/dev/null 2>&1; then
    echo -e "  ${GREEN}✔ ffmpeg is installed${NC}\n"
else
    echo -e "  ${YELLOW}⚠️  ffmpeg is not installed (required for video encoding).${NC}"
    echo -e "     Install via: sudo pacman -S ffmpeg  or  sudo apt install ffmpeg\n"
fi

REPO="Praveensenpai/emaki"
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/emaki-x86_64-linux.tar.gz"

LOCAL_DIR=""
if [ -n "${BASH_SOURCE[0]:-}" ] && [ -f "${BASH_SOURCE[0]}" ]; then
    LOCAL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)"
fi

INSTALLED_VER="latest"

if [ -n "$LOCAL_DIR" ] && [ -f "$LOCAL_DIR/Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
    VERSION=$(grep -m1 '^version' "$LOCAL_DIR/Cargo.toml" | cut -d '"' -f2 2>/dev/null || echo "latest")
    echo -e "${BLUE}📦 Local source detected. Building emaki v${VERSION} with Cargo...${NC}"
    cargo build --release --manifest-path "$LOCAL_DIR/Cargo.toml"
    cp "$LOCAL_DIR/target/release/emaki" "$BIN_DIR/emaki"
    INSTALLED_VER="v${VERSION}"
else
    LATEST_TAG=$(curl -4 -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep -o '"tag_name": "[^"]*"' | cut -d'"' -f4 || true)
    [ -z "$LATEST_TAG" ] && LATEST_TAG="latest"
    echo -e "${BLUE}📦 Downloading emaki ${LATEST_TAG} pre-compiled binary from GitHub Releases...${NC}"
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    if curl -4 -fL --connect-timeout 10 --retry 3 -sS "$RELEASE_URL" -o "$TMP_DIR/emaki.tar.gz"; then
        tar -xzf "$TMP_DIR/emaki.tar.gz" -C "$TMP_DIR"
        cp "$TMP_DIR/emaki" "$BIN_DIR/emaki"
        INSTALLED_VER="${LATEST_TAG}"
    else
        echo -e "${RED}❌ Failed to download pre-compiled release.${NC}"
        exit 1
    fi
fi

chmod +x "$BIN_DIR/emaki"

echo -e "${GREEN}✨ 絵巻 (Emaki) ${INSTALLED_VER} installed successfully to ${BIN_DIR}/emaki!${NC}\n"

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "${YELLOW}⚠️  Note: ${BIN_DIR} is not in your PATH.${NC}"
    echo -e "   Add this line to your ~/.bashrc or ~/.zshrc:"
    echo -e "   export PATH=\"\$HOME/.local/bin:\$PATH\"\n"
fi

echo -e "${BOLD}To start the daemon:${NC}"
echo -e "  emaki\n"
