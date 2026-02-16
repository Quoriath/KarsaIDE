use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AIProvider {
    #[default]
    Kilo,
    Ollama,
    OpenAI,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AIConfig {
    pub provider: AIProvider,
    pub api_key: Option<String>,
    pub base_url: String,
    pub model_name: String,
    pub custom_models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecuritySettings {
    pub auto_execute_shell: bool,
    pub auto_delete_files: bool,
    pub auto_move_files: bool,
    pub auto_save_files: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EditorConfig {
    pub font_size: u32,
    pub tab_size: u32,
    pub word_wrap: bool,
    pub theme: String,
    pub auto_save: bool,
    pub auto_save_delay: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    pub shell: Option<String>,
    pub font_size: u32,
    pub scrollback: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionData {
    pub last_workspace: Option<String>,
    pub open_files: Vec<String>,
    pub active_file: Option<String>,
    pub recent_workspaces: Vec<RecentWorkspace>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    pub path: String,
    pub name: String,
    pub last_opened: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KarsaConfig {
    #[serde(default)]
    pub ai: AIConfig,
    #[serde(default)]
    pub editor: EditorConfig,
    #[serde(default)]
    pub terminal: TerminalConfig,
    #[serde(default)]
    pub session: SessionData,
    #[serde(default)]
    pub security: SecuritySettings,
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
            auto_save: true,
            auto_save_delay: 1000,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            shell: None,
            font_size: 13,
            scrollback: 1000,
        }
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self {
            last_workspace: None,
            open_files: vec![],
            active_file: None,
            recent_workspaces: vec![],
        }
    }
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            auto_execute_shell: false,
            auto_delete_files: false,
            auto_move_files: false,
            auto_save_files: true,
        }
    }
}

impl Default for KarsaConfig {
    fn default() -> Self {
        Self {
            ai: AIConfig::default(),
            editor: EditorConfig::default(),
            terminal: TerminalConfig::default(),
            session: SessionData::default(),
            security: SecuritySettings::default(),
        }
    }
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("karsa-ide");

    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    Ok(config_dir.join("karsa_config.json"))
}

pub fn load_config() -> KarsaConfig {
    match get_config_path() {
        Ok(path) if path.exists() => {
            log::info!("Loading config from: {:?}", path);
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str(&content).ok())
                .unwrap_or_default()
        }
        Ok(path) => {
            log::info!("Config file not found at: {:?}", path);
            KarsaConfig::default()
        }
        _ => KarsaConfig::default(),
    }
}

pub fn save_config(config: &KarsaConfig) -> Result<()> {
    let path = get_config_path()?;
    let content = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    fs::write(&path, content).context("Failed to write config")?;
    log::info!("Config saved to: {:?}", path);
    Ok(())
}

#[allow(dead_code)]
pub fn config_exists() -> bool {
    get_config_path().map(|p| p.exists()).unwrap_or(false)
}

#[allow(dead_code)]
pub fn get_config_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Failed to get config directory")?
        .join("karsa-ide");

    fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

    Ok(config_dir)
}

pub fn update_session(session: SessionData) -> Result<()> {
    let mut config = load_config();
    config.session = session;
    save_config(&config)
}
