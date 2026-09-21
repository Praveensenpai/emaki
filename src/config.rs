use crate::error::{EmakiError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_session_db")]
    pub session_db: PathBuf,

    #[serde(default)]
    pub phone_number: Option<String>,

    #[serde(default)]
    pub whitelist_groups: Vec<String>,

    #[serde(default = "default_temp_dir")]
    pub temp_dir: PathBuf,

    #[serde(default = "default_cache_dir")]
    pub cache_dir: PathBuf,

    #[serde(default = "default_max_cache_gb")]
    pub max_cache_size_gb: u64,

    #[serde(default = "default_max_size_mb")]
    pub max_file_size_mb: u64,

    #[serde(default = "default_caption_prefix")]
    pub caption_prefix: String,
}

fn default_session_db() -> PathBuf {
    PathBuf::from("emaki.db")
}

fn default_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

fn default_cache_dir() -> PathBuf {
    PathBuf::from("cache")
}

fn default_max_cache_gb() -> u64 {
    5
}

fn default_max_size_mb() -> u64 {
    50
}

fn default_caption_prefix() -> String {
    "🎬 Reel via 絵巻".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            session_db: default_session_db(),
            phone_number: None,
            whitelist_groups: Vec::new(),
            temp_dir: default_temp_dir(),
            cache_dir: default_cache_dir(),
            max_cache_size_gb: default_max_cache_gb(),
            max_file_size_mb: default_max_size_mb(),
            caption_prefix: default_caption_prefix(),
        }
    }
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| EmakiError::Config(format!("Failed to parse TOML config: {e}")))?;
        Ok(config)
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        Self::load_from_file(path).unwrap_or_default()
    }

    pub fn is_group_allowed(&self, group_jid: &str) -> bool {
        if self.whitelist_groups.is_empty() {
            return true;
        }
        self.whitelist_groups.iter().any(|g| g == group_jid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.session_db, PathBuf::from("emaki.db"));
        assert_eq!(cfg.cache_dir, PathBuf::from("cache"));
        assert_eq!(cfg.max_cache_size_gb, 5);
        assert!(cfg.whitelist_groups.is_empty());
        assert!(cfg.is_group_allowed("any_group@g.us"));
    }

    #[test]
    fn test_whitelist_filter() {
        let mut cfg = Config::default();
        cfg.whitelist_groups.push("123@g.us".to_string());
        assert!(cfg.is_group_allowed("123@g.us"));
        assert!(!cfg.is_group_allowed("456@g.us"));
    }

    #[test]
    fn test_load_or_default_fallback() {
        let cfg = Config::load_or_default("non_existent_file_path.toml");
        assert_eq!(cfg.session_db, PathBuf::from("emaki.db"));
    }
}
