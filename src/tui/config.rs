#![warn(clippy::all, clippy::pedantic)]

use serde::Deserialize;
use std::path::PathBuf;

/// Default model name used when no configuration is provided
const DEFAULT_MODEL_NAME: &str = "qwen3-coder:latest";

/// Configuration file structure
#[derive(Deserialize, Default)]
struct Config {
    /// Model name to use for LLM generation
    model: Option<String>,
}

/// Get the platform-appropriate path for the config file.
/// On macOS: ~/Library/Application Support/chat/config.toml
/// On Linux: ~/.config/chat/config.toml
fn get_config_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    Some(config_dir.join("chat").join("config.toml"))
}

/// Load the model name from configuration.
/// Priority: 1. `CHAT_MODEL` env var, 2. config file, 3. default
pub fn get_model_name() -> String {
    // 1. Check environment variable first
    if let Ok(model) = std::env::var("CHAT_MODEL") {
        if !model.trim().is_empty() {
            return model.trim().to_string();
        }
    }

    // 2. Try to read from config file
    if let Some(config_path) = get_config_path() {
        if let Ok(contents) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<Config>(&contents) {
                if let Some(model) = config.model {
                    if !model.trim().is_empty() {
                        return model.trim().to_string();
                    }
                }
            }
        }
    }

    // 3. Fall back to default
    DEFAULT_MODEL_NAME.to_string()
}

/// Get the platform-appropriate path for the history file.
/// On macOS: ~/Library/Caches/chat/history
/// On Linux: ~/.cache/chat/history
/// Creates parent directories if they don't exist.
///
pub fn get_history_path() -> Option<PathBuf> {
    let cache_dir = dirs::cache_dir()?;
    let history_dir = cache_dir.join("chat");

    // Create parent directories if they don't exist
    if !history_dir.exists() {
        std::fs::create_dir_all(&history_dir).ok()?;
    }

    Some(history_dir.join("history"))
}
