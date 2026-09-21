use crate::error::{EmakiError, Result};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tracing::{debug, info};

pub struct DownloadedVideo {
    pub bytes: Vec<u8>,
}

pub struct ReelDownloader {
    temp_dir: PathBuf,
    max_size_bytes: u64,
}

impl ReelDownloader {
    pub fn new(temp_dir: impl AsRef<Path>, max_size_mb: u64) -> Self {
        Self {
            temp_dir: temp_dir.as_ref().to_path_buf(),
            max_size_bytes: max_size_mb * 1024 * 1024,
        }
    }

    pub async fn download(&self, reel_id: &str, reel_url: &str) -> Result<DownloadedVideo> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let output_path = self.temp_dir.join(format!("emaki_{reel_id}_{ts}.mp4"));

        info!("Starting download for reel: {reel_id} -> {output_path:?}");

        let status = Command::new("yt-dlp")
            .arg("--no-warnings")
            .arg("--no-playlist")
            .arg("-f")
            .arg("b[ext=mp4]/best[ext=mp4]/best")
            .arg("-o")
            .arg(&output_path)
            .arg(reel_url)
            .status()
            .await
            .map_err(|e| EmakiError::Downloader(format!("Failed to execute yt-dlp: {e}")))?;

        if !status.success() {
            return Err(EmakiError::Downloader(format!(
                "yt-dlp exited with non-zero status: {status:?}"
            )));
        }

        if !output_path.exists() {
            return Err(EmakiError::Downloader(
                "Target output video file not found after download".to_string(),
            ));
        }

        let meta = tokio::fs::metadata(&output_path).await?;
        if meta.len() > self.max_size_bytes {
            let _ = tokio::fs::remove_file(&output_path).await;
            return Err(EmakiError::Downloader(format!(
                "Video file exceeds size limit ({} MB)",
                self.max_size_bytes / (1024 * 1024)
            )));
        }

        let bytes = tokio::fs::read(&output_path).await?;
        debug!("Downloaded {} bytes for reel: {reel_id}", bytes.len());

        let _ = tokio::fs::remove_file(&output_path).await;

        Ok(DownloadedVideo { bytes })
    }
}
