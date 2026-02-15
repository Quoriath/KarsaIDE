use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    Kilo,
    Ollama,
    OpenAI,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model_name: String,
    pub custom_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    pub font_size: u32,
    pub tab_size: u32,
    pub word_wrap: bool,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarsaConfig {
    pub ai: AIConfig,
    pub editor: EditorConfig,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::Kilo,
            api_key: None,
            base_url: "https://api.kilo.ai/api/gateway/chat/completions".to_string(),
            model_name: "minimax/minimax-m2.5:free".to_string(),
            custom_models: vec![],
        }
    }
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_size: 14,
            tab_size: 2,
            word_wrap: true,
            theme: "dark".to_string(),
        }
    }
}

impl Default for KarsaConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            editor: EditorConfig::default(),
        }
    }
}

fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("karsa-ide");

    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    
    Ok(config_dir.join("karsa_config.json"))
}

pub fn load_config() -> KarsaConfig {
    match get_config_path() {
        Ok(path) if path.exists() => {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        }
        _ => KarsaConfig::default(),
    }
}

pub fn save_config(config: &KarsaConfig) -> Result<()> {
    let path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;
    fs::write(&path, content)
        .context("Failed to write config")?;
    Ok(())
}

pub fn config_exists() -> bool {
    get_config_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

#[allow(dead_code)]
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("karsa-ide");
    
    fs::create_dir_all(&config_dir)
        .context("Failed to create config directory")?;
    
    Ok(config_dir)
}
