use super::app_config::Config;
use anyhow::Result;
use parking_lot::RwLock;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;

impl Config {
    /// Get the configuration file path based on the platform
    pub fn path() -> PathBuf {
        const FILENAME: &str = "config.json";
        if let Some(mut path) = dirs::config_dir() {
            path.push("arto");
            path.push(FILENAME);
            return path;
        }

        // Fallback to home directory
        if let Some(mut path) = dirs::home_dir() {
            path.push(".arto");
            path.push(FILENAME);
            return path;
        }

        PathBuf::from(FILENAME)
    }

    /// Load configuration from file or return default configuration
    pub fn load() -> Result<Self> {
        let path = Self::path();

        if !path.exists() {
            return Ok(Config {
                keybindings: crate::keybindings::default_bindings(),
                ..Default::default()
            });
        }

        let content = fs::read_to_string(&path)?;
        let raw: serde_json::Value = serde_json::from_str(&content)?;
        let mut config: Config = serde_json::from_value(raw.clone())?;

        if should_populate_default_keybindings(&raw, &config) {
            config.keybindings = crate::keybindings::default_bindings();
        }

        tracing::debug!(path = %path.display(), "Configuration loaded");

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::path();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self)?;
        fs::write(&path, content)?;

        tracing::debug!(path = %path.display(), "Configuration saved");

        Ok(())
    }
}

fn should_populate_default_keybindings(raw: &serde_json::Value, config: &Config) -> bool {
    let has_keybindings_field = raw
        .as_object()
        .is_some_and(|obj| obj.contains_key("keybindings"));

    !has_keybindings_field && config.keybindings.is_empty()
}

/// Global configuration instance
pub static CONFIG: LazyLock<RwLock<Config>> = LazyLock::new(|| {
    let config = Config::load().unwrap_or_default();
    RwLock::new(config)
});
