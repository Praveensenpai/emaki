use crate::config::Config;
use crate::domain::{ExtractedReel, ReelExtractor};
use crate::error::{EmakiError, Result};
use crate::infra::downloader::ReelDownloader;
use crate::infra::qr::render_terminal_qr;
use std::sync::Arc;
use tracing::{error, info, warn};
use whatsapp_rust::download::MediaType;
use whatsapp_rust::media::{video_message, VideoOptions};
use whatsapp_rust::prelude::*;
use whatsapp_rust::upload::UploadOptions;

pub struct WhatsAppBot {
    config: Config,
    downloader: Arc<ReelDownloader>,
}

impl WhatsAppBot {
    pub fn new(config: Config) -> Self {
        let downloader = Arc::new(ReelDownloader::new(
            &config.temp_dir,
            config.max_file_size_mb,
        ));
        Self { config, downloader }
    }

    pub async fn start(&self) -> Result<()> {
        let db_path = self.config.session_db.to_str().unwrap_or("emaki.db");

        let store = SqliteStore::new(db_path)
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Database initialization failed: {e}")))?;

        info!("SQLite session store initialized at: {db_path}");

        let downloader = Arc::clone(&self.downloader);
        let config = self.config.clone();

        let bot = Bot::builder()
            .with_backend(store)
            .on_qr_code(|code, timeout| async move {
                if let Err(e) = render_terminal_qr(&code, timeout.as_secs()) {
                    error!("Failed to render QR code: {e}");
                }
            })
            .on_connected(|_client| async {
                info!("✨ 絵巻 (Emaki) daemon connected and active!");
            })
            .on_logged_out(|_info| async {
                warn!("⚠️ Session logged out. Please restart and re-scan QR code.");
            })
            .on_message(move |ctx| {
                let downloader = Arc::clone(&downloader);
                let config = config.clone();
                async move {
                    Self::process_message(ctx, downloader, config).await;
                }
            })
            .build()
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Failed to build WhatsApp bot: {e}")))?;

        info!("Starting bot event loop...");
        bot.run().await;

        Ok(())
    }

    async fn process_message(ctx: MessageContext, downloader: Arc<ReelDownloader>, config: Config) {
        if ctx.info.source.is_from_me {
            return;
        }

        let chat_str = ctx.info.source.chat.to_string();
        if !config.is_group_allowed(&chat_str) {
            return;
        }

        let Some(text) = extract_message_text(&ctx.message) else {
            return;
        };

        let reels = ReelExtractor::extract_all(text);
        if reels.is_empty() {
            return;
        }

        info!(
            "Found {} Instagram Reel(s) from chat {}",
            reels.len(),
            chat_str
        );

        for reel in reels {
            Self::download_and_send(&ctx, &reel, &downloader, &config.caption_prefix).await;
        }
    }

    async fn download_and_send(
        ctx: &MessageContext,
        reel: &ExtractedReel,
        downloader: &ReelDownloader,
        caption_prefix: &str,
    ) {
        let _ = ctx.react("⏳").await;

        let video = match downloader.download(&reel.id, &reel.canonical_url).await {
            Ok(v) => v,
            Err(e) => {
                error!("Download failed for reel {}: {e}", reel.id);
                let _ = ctx.react("❌").await;
                return;
            }
        };

        let upload_res = ctx
            .client
            .upload(video.bytes, MediaType::Video, UploadOptions::default())
            .await;

        let upload = match upload_res {
            Ok(u) => u,
            Err(e) => {
                error!("Upload failed for reel {}: {e}", reel.id);
                let _ = ctx.react("❌").await;
                return;
            }
        };

        let caption = format!("{caption_prefix} {}", reel.canonical_url);
        let opts = VideoOptions {
            caption: Some(caption),
            ..Default::default()
        };

        let msg = video_message(upload, opts);

        if let Err(e) = ctx.send_message(msg).await {
            error!("Failed to send video message: {e}");
            let _ = ctx.react("❌").await;
        } else {
            let _ = ctx.react("✅").await;
            info!("Successfully delivered reel {} to group", reel.id);
        }
    }
}

fn extract_message_text(msg: &wa::Message) -> Option<&str> {
    if let Some(conv) = &msg.conversation {
        return Some(conv.as_str());
    }
    if let Some(ext) = msg.extended_text_message.as_option() {
        if let Some(text) = &ext.text {
            return Some(text.as_str());
        }
    }
    if let Some(img) = msg.image_message.as_option() {
        if let Some(caption) = &img.caption {
            return Some(caption.as_str());
        }
    }
    if let Some(vid) = msg.video_message.as_option() {
        if let Some(caption) = &vid.caption {
            return Some(caption.as_str());
        }
    }
    None
}
