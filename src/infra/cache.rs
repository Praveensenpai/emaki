use crate::error::Result;
use crate::infra::downloader::DownloadedVideo;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{debug, info, warn};

#[derive(Serialize, Deserialize)]
struct CacheMeta {
    uploader: Option<String>,
    description: Option<String>,
}

pub struct ReelCache {
    cache_dir: PathBuf,
    max_bytes: u64,
}

impl ReelCache {
    pub fn new(cache_dir: impl AsRef<Path>, max_size_gb: u64) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            max_bytes: max_size_gb * 1024 * 1024 * 1024,
        }
    }

    pub async fn get(&self, reel_id: &str) -> Option<DownloadedVideo> {
        let video_path = self.video_path(reel_id);
        if !video_path.is_file() {
            debug!("Cache miss (no video file) for reel: {reel_id}");
            return None;
        }

        let bytes = match tokio::fs::read(&video_path).await {
            Ok(b) if !b.is_empty() => b,
            _ => {
                debug!("Cache invalid (empty or unreadable) for reel: {reel_id}");
                return None;
            }
        };

        let (uploader, description) = self.read_meta(reel_id).await;
        debug!("Cache hit ({} bytes) for reel: {reel_id}", bytes.len());

        Some(DownloadedVideo {
            bytes,
            uploader,
            description,
        })
    }

    pub async fn put(&self, reel_id: &str, video: &DownloadedVideo) -> Result<()> {
        if let Err(e) = tokio::fs::create_dir_all(&self.cache_dir).await {
            warn!("Failed to create cache dir: {e}");
            return Ok(());
        }

        let video_path = self.video_path(reel_id);
        if let Err(e) = tokio::fs::write(&video_path, &video.bytes).await {
            warn!("Failed to write video cache for {reel_id}: {e}");
            return Ok(());
        }

        let meta = CacheMeta {
            uploader: video.uploader.clone(),
            description: video.description.clone(),
        };
        if let Ok(meta_json) = serde_json::to_string(&meta) {
            let meta_path = self.meta_path(reel_id);
            let _ = tokio::fs::write(&meta_path, meta_json).await;
        }

        info!("Cached reel: {reel_id} ({} bytes)", video.bytes.len());
        self.prune_if_needed().await;

        Ok(())
    }

    async fn read_meta(&self, reel_id: &str) -> (Option<String>, Option<String>) {
        let meta_path = self.meta_path(reel_id);
        if let Ok(content) = tokio::fs::read_to_string(&meta_path).await {
            if let Ok(meta) = serde_json::from_str::<CacheMeta>(&content) {
                return (meta.uploader, meta.description);
            }
        }
        (None, None)
    }

    async fn prune_if_needed(&self) {
        let mut entries = Vec::new();
        let mut total_size = 0u64;

        let Ok(mut read_dir) = tokio::fs::read_dir(&self.cache_dir).await else {
            return;
        };

        while let Ok(Some(entry)) = read_dir.next_entry().await {
            if let Ok(meta) = entry.metadata().await {
                if meta.is_file() {
                    let size = meta.len();
                    total_size += size;
                    let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    entries.push((entry.path(), size, mtime));
                }
            }
        }

        if total_size <= self.max_bytes {
            return;
        }

        info!(
            "Cache size ({} MB) exceeds limit ({} MB). Pruning oldest files...",
            total_size / (1024 * 1024),
            self.max_bytes / (1024 * 1024)
        );

        entries.sort_by_key(|e| e.2);

        let target_size = (self.max_bytes as f64 * 0.85) as u64;
        for (path, size, _) in entries {
            if total_size <= target_size {
                break;
            }
            if tokio::fs::remove_file(&path).await.is_ok() {
                total_size = total_size.saturating_sub(size);
            }
        }
    }

    fn video_path(&self, reel_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{reel_id}.mp4"))
    }

    fn meta_path(&self, reel_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{reel_id}.json"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_miss_on_empty() {
        let dir = std::env::temp_dir().join("emaki_test_empty");
        let cache = ReelCache::new(&dir, 1);
        let res = cache.get("non_existent").await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_cache_put_and_get() {
        let dir = std::env::temp_dir().join("emaki_test_put_get");
        let _ = tokio::fs::remove_dir_all(&dir).await;

        let cache = ReelCache::new(&dir, 1);
        let sample_video = DownloadedVideo {
            bytes: b"video-content".to_vec(),
            uploader: Some("paisen".into()),
            description: Some("test caption".into()),
        };

        cache.put("reel123", &sample_video).await.unwrap();

        let cached = cache.get("reel123").await;
        assert!(cached.is_some());
        let cached = cached.unwrap();
        assert_eq!(cached.bytes, b"video-content");
        assert_eq!(cached.uploader.as_deref(), Some("paisen"));

        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
