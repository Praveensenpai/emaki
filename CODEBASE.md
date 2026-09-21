# CODEBASE.md: 絵巻 (Emaki) Semantic Digest

> **Notice**: This file is an AI-optimized semantic index. Do not write narrative prose. Keep token density high.

## 1. System Topology & Data Flow
```text
WhatsApp Group Message 
       │
       ▼
[WhatsAppBot (whatsapp-rust Tokio)] ──(Extract Message Text)──> [ReelExtractor (domain/reel)]
       │                                                                  │
       │                                                                  ▼
       ├─── (Reaction "⏳")                                       ExtractedReel { id, canonical_url }
       │                                                                  │
       ▼                                                                  ▼
[ctx.client.upload] <─── DownloadedVideo { bytes } <─── [ReelDownloader (yt-dlp subprocess)]
       │
       ▼
[whatsapp_rust::media::video_message] ──> [ctx.send_message] ──> Group Chat (Reaction "✅")
```

## 2. Global Constraints & Architecture Patterns
- **Primary Language & Edition**: Rust 2021 edition (stable toolchain >= 1.94).
- **Architectural Paradigm**: Role-based clean architecture (`domain/`, `infra/`, `config.rs`, `error.rs`).
- **Hard Constraints**: <400 lines/file, <60 lines/fn, max 4 parameters, zero production `unwrap()`/`expect()`, zero dead code, 0 compiler/clippy warnings.
- **Resource Footprint**: ~10–20MB idle RAM, sub-second execution, zero persistent disk overhead (temp videos dropped immediately).

## 3. Module & Interface Skeleton

### `src/error.rs` (Role: domain, Lines: 18)
- **Responsibility**: Centralized error enum and result alias across the application.
- **Imports**: `thiserror::Error`, `std::io::Error`.
- **Types & Enums**:
  ```rust
  pub enum EmakiError {
      Io(std::io::Error),
      Config(String),
      Downloader(String),
      WhatsApp(String),
  }
  pub type Result<T> = std::result::Result<T, EmakiError>;
  ```

### `src/config.rs` (Role: infra, Lines: 100)
- **Responsibility**: TOML configuration deserialization and group whitelist verification.
- **Imports**: `crate::error::{EmakiError, Result}`, `serde::{Deserialize, Serialize}`, `std::path::{Path, PathBuf}`.
- **Types & Enums**:
  ```rust
  pub struct Config {
      pub session_db: PathBuf,
      pub phone_number: Option<String>,
      pub whitelist_groups: Vec<String>,
      pub temp_dir: PathBuf,
      pub max_file_size_mb: u64,
      pub caption_prefix: String,
  }
  impl Config {
      pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self>;
      pub fn load_or_default(path: impl AsRef<Path>) -> Self;
      pub fn is_group_allowed(&self, group_jid: &str) -> bool;
  }
  ```

### `src/domain.rs` (Role: domain, Lines: 3)
- **Responsibility**: Re-exports pure domain models and extractors.
- **Exports**: `ExtractedReel`, `ReelExtractor`.

### `src/domain/reel.rs` (Role: domain, Lines: 67)
- **Responsibility**: Regular expression parsing and canonicalization of Instagram reel URLs.
- **Imports**: `regex::Regex`, `std::sync::LazyLock`, `std::collections::HashSet`.
- **Types & Enums**:
  ```rust
  pub struct ExtractedReel {
      pub id: String,
      pub canonical_url: String,
  }
  pub struct ReelExtractor;
  impl ReelExtractor {
      pub fn extract_all(text: &str) -> Vec<ExtractedReel>;
  }
  ```

### `src/infra.rs` (Role: infra, Lines: 5)
- **Responsibility**: Declares and re-exports infrastructure drivers.
- **Exports**: `WhatsAppBot`.

### `src/infra/qr.rs` (Role: infra, Lines: 20)
- **Responsibility**: Terminal QR code generation using `fast_qr` for WhatsApp Web linking.
- **Imports**: `crate::error::{EmakiError, Result}`, `fast_qr::QRBuilder`, `tracing::info`.
- **Functions**:
  ```rust
  pub fn render_terminal_qr(code: &str, timeout_secs: u64) -> Result<()>;
  ```

### `src/infra/downloader.rs` (Role: infra, Lines: 73)
- **Responsibility**: Asynchronous invocation of `yt-dlp` CLI, size boundary validation, and disk cleanup.
- **Imports**: `crate::error::{EmakiError, Result}`, `tokio::process::Command`, `std::path::{Path, PathBuf}`.
- **Types & Enums**:
  ```rust
  pub struct DownloadedVideo {
      pub bytes: Vec<u8>,
  }
  pub struct ReelDownloader {
      temp_dir: PathBuf,
      max_size_bytes: u64,
  }
  impl ReelDownloader {
      pub fn new(temp_dir: impl AsRef<Path>, max_size_mb: u64) -> Self;
      pub async fn download(&self, reel_id: &str, reel_url: &str) -> Result<DownloadedVideo>;
  }
  ```

### `src/infra/whatsapp.rs` (Role: infra, Lines: 168)
- **Responsibility**: WhatsApp bot lifecycle, event binding, reaction dispatch, and encrypted media upload.
- **Imports**: `whatsapp_rust::prelude::*`, `whatsapp_rust::download::MediaType`, `whatsapp_rust::media::{video_message, VideoOptions}`, `whatsapp_rust::upload::UploadOptions`.
- **Types & Enums**:
  ```rust
  pub struct WhatsAppBot {
      config: Config,
      downloader: Arc<ReelDownloader>,
  }
  impl WhatsAppBot {
      pub fn new(config: Config) -> Self;
      pub async fn start(&self) -> Result<()>;
  }
  ```

### `src/main.rs` (Role: entrypoint, Lines: 43)
- **Responsibility**: Binary bootstrap, logging subscriber configuration, and graceful exit orchestration.
- **Imports**: `tracing_subscriber::{fmt, EnvFilter}`, `infra::WhatsAppBot`, `config::Config`.
- **Functions**:
  ```rust
  #[tokio::main]
  async fn main() -> Result<()>;
  fn print_banner();
  ```
