use crate::config::Config;
use crate::domain::{format_caption, ExtractedReel, ReelExtractor};
use crate::error::{EmakiError, Result};
use crate::infra::cache::ReelCache;
use crate::infra::downloader::ReelDownloader;
use crate::infra::qr::render_terminal_qr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{channel, Sender};
use tracing::{error, info, warn};
use whatsapp_rust::download::MediaType;
use whatsapp_rust::media::{video_message, VideoOptions};
use whatsapp_rust::prelude::*;
use whatsapp_rust::upload::UploadOptions;

struct ReelJob {
    ctx: MessageContext,
    reel: ExtractedReel,
}

pub struct WhatsAppBot {
    config: Config,
    downloader: Arc<ReelDownloader>,
    cache: Arc<ReelCache>,
}

impl WhatsAppBot {
    pub fn new(config: Config) -> Self {
        let downloader = Arc::new(ReelDownloader::new(
            &config.temp_dir,
            config.max_file_size_mb,
        ));
        let cache = Arc::new(ReelCache::new(&config.cache_dir, config.max_cache_size_gb));
        Self {
            config,
            downloader,
            cache,
        }
    }

    pub async fn check_login_status(session_db: &Path) -> Result<Option<String>> {
        if !session_db.exists() {
            return Ok(None);
        }
        let db_str = session_db.to_str().unwrap_or("emaki.db");
        let store = SqliteStore::new(db_str)
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Database error: {e}")))?;

        if let Ok(Some(dev)) = store.load_device_data_for_device(1).await {
            if let Some(pn) = dev.pn {
                return Ok(Some(pn.to_string()));
            }
            if let Some(lid) = dev.lid {
                return Ok(Some(lid.to_string()));
            }
        }
        Ok(None)
    }

    pub async fn start_login(&self) -> Result<()> {
        let db_path = self.config.session_db.to_str().unwrap_or("emaki.db");
        let store = SqliteStore::new(db_path)
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Database init failed: {e}")))?;

        info!("Starting interactive WhatsApp pairing...");

        let bot = Bot::builder()
            .with_backend(store)
            .on_qr_code(|code, timeout| async move {
                if let Err(e) = render_terminal_qr(&code, timeout.as_secs()) {
                    error!("Failed to render QR code: {e}");
                }
            })
            .on_connected(|client| async move {
                info!("══════════════════════════════════════════════════════════");
                info!("🎉 WhatsApp linked successfully!");
                info!("📁 Session credentials saved to emaki.db");
                info!("🚀 You can now start the daemon: emaki daemon");
                info!("══════════════════════════════════════════════════════════");
                client.disconnect().await;
            })
            .build()
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Failed to build bot: {e}")))?;

        bot.run().await;
        Ok(())
    }

    pub async fn start_daemon(&self) -> Result<()> {
        let db_path = self.config.session_db.to_str().unwrap_or("emaki.db");

        match Self::check_login_status(&self.config.session_db).await? {
            Some(jid) => info!("Verified authenticated session for device: {jid}"),
            None => {
                return Err(EmakiError::WhatsApp(
                    "No authenticated session found in emaki.db. Please run 'emaki login' first to pair your WhatsApp account!".to_string(),
                ));
            }
        }

        let store = SqliteStore::new(db_path)
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Database error: {e}")))?;

        let (job_tx, mut job_rx) = channel::<ReelJob>(100);
        let worker_downloader = Arc::clone(&self.downloader);
        let worker_cache = Arc::clone(&self.cache);

        tokio::spawn(async move {
            info!("Reel queue worker active (3s pacing, 5GB LRU cache)");
            while let Some(job) = job_rx.recv().await {
                Self::download_and_send(&job.ctx, &job.reel, &worker_downloader, &worker_cache)
                    .await;
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        });

        let config = self.config.clone();

        let bot = Bot::builder()
            .with_backend(store)
            .on_connected(|_client| async {
                info!("✨ 絵巻 (Emaki) daemon connected and relaying reels!");
            })
            .on_logged_out(|_info| async {
                warn!("⚠️ Session logged out. Run 'emaki login' to re-authenticate.");
            })
            .on_message(move |ctx| {
                let job_tx = job_tx.clone();
                let config = config.clone();
                async move {
                    Self::process_message(ctx, &job_tx, &config).await;
                }
            })
            .build()
            .await
            .map_err(|e| EmakiError::WhatsApp(format!("Failed to build bot: {e}")))?;

        bot.run().await;
        Ok(())
    }

    async fn process_message(ctx: MessageContext, job_tx: &Sender<ReelJob>, config: &Config) {
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
            "Found {} Instagram Reel(s) in chat {}. Enqueueing...",
            reels.len(),
            chat_str
        );

        for reel in reels {
            let job = ReelJob {
                ctx: ctx.clone(),
                reel,
            };
            if let Err(e) = job_tx.send(job).await {
                error!("Failed to enqueue reel job: {e}");
            }
        }
    }

    async fn download_and_send(
        ctx: &MessageContext,
        reel: &ExtractedReel,
        downloader: &ReelDownloader,
        cache: &ReelCache,
    ) {
        let video = if let Some(cached) = cache.get(&reel.id).await {
            info!("Serving reel {} directly from 5GB cache", reel.id);
            cached
        } else {
            match downloader.download(&reel.id, &reel.canonical_url).await {
                Ok(v) => {
                    let _ = cache.put(&reel.id, &v).await;
                    v
                }
                Err(e) => {
                    error!("Download failed for reel after retries {}: {e}", reel.id);
                    return;
                }
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
                return;
            }
        };

        let caption = format_caption(
            video.uploader.as_deref(),
            video.description.as_deref(),
            &reel.canonical_url,
        );
        let opts = VideoOptions {
            caption: Some(caption),
            ..Default::default()
        };

        let msg = video_message(upload, opts);

        if let Err(e) = ctx.send_message(msg).await {
            error!("Failed to send video message: {e}");
        } else {
            info!("Successfully delivered reel {} to chat", reel.id);
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
