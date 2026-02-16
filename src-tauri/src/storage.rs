use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFolder {
    pub path: String,
    pub name: String,
    pub last_opened: i64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub workspace: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStorage {
    pub last_workspace: Option<String>,
    pub recent_folders: Vec<RecentFolder>,
    pub chat_sessions: Vec<ChatSession>,
}

impl Default for AppStorage {
    fn default() -> Self {
        Self {
            last_workspace: None,
            recent_folders: Vec::new(),
            chat_sessions: Vec::new(),
        }
    }
}

pub struct StorageManager {
    storage_path: PathBuf,
}

impl StorageManager {
    pub fn new() -> Result<Self> {
        let storage_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
            .join("karsa-ide");
        
        std::fs::create_dir_all(&storage_dir)?;
        
        Ok(Self {
            storage_path: storage_dir.join("storage.json"),
        })
    }
    
    pub fn load(&self) -> Result<AppStorage> {
        if !self.storage_path.exists() {
            return Ok(AppStorage::default());
        }
        
        let content = std::fs::read_to_string(&self.storage_path)?;
        let storage: AppStorage = serde_json::from_str(&content)?;
        Ok(storage)
    }
    
    pub fn save(&self, storage: &AppStorage) -> Result<()> {
        let content = serde_json::to_string_pretty(storage)?;
        std::fs::write(&self.storage_path, content)?;
        Ok(())
    }
    
    pub fn add_recent_folder(&self, path: String, name: String) -> Result<()> {
        let mut storage = self.load()?;
        let timestamp = chrono::Utc::now().timestamp();
        
        // Remove if already exists
        storage.recent_folders.retain(|f| f.path != path);
        
        // Add to front
        storage.recent_folders.insert(0, RecentFolder {
            path,
            name,
            last_opened: timestamp,
        });
        
        // Keep only last 10
        storage.recent_folders.truncate(10);
        
        self.save(&storage)?;
        Ok(())
    }
    
    pub fn set_last_workspace(&self, path: String) -> Result<()> {
        let mut storage = self.load()?;
        storage.last_workspace = Some(path);
        self.save(&storage)?;
        Ok(())
    }
    
    pub fn get_last_workspace(&self) -> Result<Option<String>> {
        let storage = self.load()?;
        Ok(storage.last_workspace)
    }
    
    pub fn save_chat_session(&self, session: ChatSession) -> Result<()> {
        let mut storage = self.load()?;
        
        // Remove old session with same ID
        storage.chat_sessions.retain(|s| s.id != session.id);
        
        // Add new session
        storage.chat_sessions.push(session);
        
        // Keep only last 50 sessions
        storage.chat_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        storage.chat_sessions.truncate(50);
        
        self.save(&storage)?;
        Ok(())
    }
    
    pub fn get_chat_sessions(&self, workspace: Option<String>) -> Result<Vec<ChatSession>> {
        let storage = self.load()?;
        
        let sessions: Vec<ChatSession> = if let Some(ws) = workspace {
            storage.chat_sessions.into_iter()
                .filter(|s| s.workspace.as_ref() == Some(&ws))
                .collect()
        } else {
            storage.chat_sessions
        };
        
        Ok(sessions)
    }
}
