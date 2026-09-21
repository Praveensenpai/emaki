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

pub fn default_emaki_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".emaki")
    } else {
        PathBuf::from(".emaki")
    }
}

fn default_session_db() -> PathBuf {
    default_emaki_dir().join("emaki.db")
}

fn default_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

fn default_cache_dir() -> PathBuf {
    default_emaki_dir().join("cache")
}

fn default_max_cache_gb() -> u64 {
    5
}

fn default_max_size_mb() -> u64 {
    500
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

    pub fn find_config_path() -> Option<PathBuf> {
        let local = PathBuf::from("emaki.toml");
        if local.exists() {
            return Some(local);
        }
        let home_cfg = default_emaki_dir().join("emaki.toml");
        if home_cfg.exists() {
            return Some(home_cfg);
        }
        None
    }

    pub fn load_auto() -> Self {
        if let Some(path) = Self::find_config_path() {
            Self::load_or_default(path)
        } else {
            Self::default()
        }
    }

    pub fn active_config_path() -> PathBuf {
        Self::find_config_path().unwrap_or_else(|| default_emaki_dir().join("emaki.toml"))
    }

    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| EmakiError::Config(format!("Failed to serialize TOML config: {e}")))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "max_file_size_mb" => {
                self.max_file_size_mb = value.parse::<u64>().map_err(|_| {
                    EmakiError::Config("Value must be a positive integer (MB)".into())
                })?;
            }
            "max_cache_size_gb" => {
                self.max_cache_size_gb = value.parse::<u64>().map_err(|_| {
                    EmakiError::Config("Value must be a positive integer (GB)".into())
                })?;
            }
            "caption_prefix" => {
                self.caption_prefix = value.to_string();
            }
            "temp_dir" => {
                self.temp_dir = PathBuf::from(value);
            }
            "cache_dir" => {
                self.cache_dir = PathBuf::from(value);
            }
            "session_db" => {
                self.session_db = PathBuf::from(value);
            }
            unknown => {
                return Err(EmakiError::Config(format!(
                    "Unknown key: '{unknown}'. Valid keys: max_file_size_mb, max_cache_size_gb, caption_prefix, temp_dir, cache_dir, session_db"
                )));
            }
        }
        Ok(())
    }

    pub fn add_whitelist_group(&mut self, jid: String) -> bool {
        if !self.whitelist_groups.contains(&jid) {
            self.whitelist_groups.push(jid);
            true
        } else {
            false
        }
    }

    pub fn remove_whitelist_group(&mut self, jid: &str) -> bool {
        let orig_len = self.whitelist_groups.len();
        self.whitelist_groups.retain(|g| g != jid);
        self.whitelist_groups.len() < orig_len
    }

    pub fn ensure_directories(&self) -> Result<()> {
        if let Some(parent) = self.session_db.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::create_dir_all(&self.cache_dir)?;

        let target_cfg = Self::active_config_path();
        if !target_cfg.exists() {
            let _ = self.save_to_file(&target_cfg);
        }

        let default_db = default_session_db();
        if self.session_db == default_db
            && !self.session_db.exists()
            && Path::new("emaki.db").exists()
        {
            let _ = std::fs::copy("emaki.db", &self.session_db);
        }
        Ok(())
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
        let expected_dir = default_emaki_dir();
        assert_eq!(cfg.session_db, expected_dir.join("emaki.db"));
        assert_eq!(cfg.cache_dir, expected_dir.join("cache"));
        assert_eq!(cfg.max_cache_size_gb, 5);
        assert_eq!(cfg.max_file_size_mb, 500);
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
        assert_eq!(cfg.session_db, default_emaki_dir().join("emaki.db"));
    }

    #[test]
    fn test_set_value_and_whitelist_helpers() {
        let mut cfg = Config::default();
        cfg.set_value("max_file_size_mb", "250").unwrap();
        assert_eq!(cfg.max_file_size_mb, 250);

        cfg.set_value("caption_prefix", "Custom").unwrap();
        assert_eq!(cfg.caption_prefix, "Custom");

        assert!(cfg.set_value("max_file_size_mb", "invalid").is_err());
        assert!(cfg.set_value("non_existent_key", "val").is_err());

        assert!(cfg.add_whitelist_group("grp1@g.us".into()));
        assert!(!cfg.add_whitelist_group("grp1@g.us".into()));
        assert!(cfg.is_group_allowed("grp1@g.us"));
        assert!(cfg.remove_whitelist_group("grp1@g.us"));
        assert!(!cfg.remove_whitelist_group("grp1@g.us"));
    }
}
