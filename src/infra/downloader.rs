use crate::error::{EmakiError, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tracing::{debug, info, warn};

pub struct DownloadedVideo {
    pub bytes: Vec<u8>,
    pub uploader: Option<String>,
    pub description: Option<String>,
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
        const MAX_RETRIES: usize = 3;
        let mut last_err = EmakiError::Downloader("Unknown error".to_string());

        for attempt in 1..=MAX_RETRIES {
            info!("Download attempt {attempt}/{MAX_RETRIES} for reel: {reel_id}");
            match self.download_once(reel_id, reel_url).await {
                Ok(video) => return Ok(video),
                Err(e) => {
                    warn!("Attempt {attempt} failed for {reel_id}: {e}");
                    last_err = e;
                    if attempt < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_secs(2 * attempt as u64)).await;
                    }
                }
            }
        }

        Err(last_err)
    }

    async fn download_once(&self, reel_id: &str, reel_url: &str) -> Result<DownloadedVideo> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let base_path = self.temp_dir.join(format!("emaki_{reel_id}_{ts}"));
        let output_tmpl = format!("{}.%(ext)s", base_path.display());
        let video_path = PathBuf::from(format!("{}.mp4", base_path.display()));
        let info_path = PathBuf::from(format!("{}.info.json", base_path.display()));

        let status = Command::new("yt-dlp")
            .arg("--no-warnings")
            .arg("--no-playlist")
            .arg("--write-info-json")
            .arg("-f")
            .arg("b[ext=mp4]/best[ext=mp4]/best")
            .arg("-o")
            .arg(&output_tmpl)
            .arg(reel_url)
            .status()
            .await
            .map_err(|e| EmakiError::Downloader(format!("Failed to execute yt-dlp: {e}")))?;

        if !status.success() {
            let _ = tokio::fs::remove_file(&info_path).await;
            return Err(EmakiError::Downloader(format!(
                "yt-dlp exited with status: {status:?}"
            )));
        }

        let (uploader, description) = Self::read_and_cleanup_metadata(&info_path).await;
        let bytes = self.read_and_cleanup_video(&video_path).await?;
        debug!("Downloaded {} bytes for reel: {reel_id}", bytes.len());

        Ok(DownloadedVideo {
            bytes,
            uploader,
            description,
        })
    }

    async fn read_and_cleanup_metadata(info_path: &Path) -> (Option<String>, Option<String>) {
        if let Ok(content) = tokio::fs::read_to_string(info_path).await {
            let _ = tokio::fs::remove_file(info_path).await;
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                let uploader = v.get("uploader").and_then(|u| u.as_str()).map(String::from);
                let description = v
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(String::from);
                return (uploader, description);
            }
        }
        (None, None)
    }

    async fn read_and_cleanup_video(&self, video_path: &Path) -> Result<Vec<u8>> {
        if !video_path.exists() {
            return Err(EmakiError::Downloader(
                "Target output video file not found after download".to_string(),
            ));
        }

        let meta = tokio::fs::metadata(video_path).await?;
        if meta.len() > self.max_size_bytes {
            let _ = tokio::fs::remove_file(video_path).await;
            return Err(EmakiError::Downloader(format!(
                "Video exceeds size limit ({} MB)",
                self.max_size_bytes / (1024 * 1024)
            )));
        }

        let bytes = tokio::fs::read(video_path).await?;
        let _ = tokio::fs::remove_file(video_path).await;
        Ok(bytes)
    }
}
