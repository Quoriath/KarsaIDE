use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CachedModels {
    pub models: Vec<crate::ai_client::ModelInfo>,
    pub timestamp: i64,
}

pub struct Cache;

impl Cache {
    fn get_cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get cache directory"))?
            .join("karsa-ide");
        
        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir.join("kilo_models.json"))
    }
    
    pub fn get_models() -> Option<Vec<crate::ai_client::ModelInfo>> {
        let path = Self::get_cache_path().ok()?;
        if !path.exists() {
            return None;
        }
        
        let content = fs::read_to_string(&path).ok()?;
        let cached: CachedModels = serde_json::from_str(&content).ok()?;
        
        // Check if cache is still valid (1 hour)
        let now = chrono::Utc::now().timestamp();
        if now - cached.timestamp < 3600 {
            Some(cached.models)
        } else {
            None
        }
    }
    
    pub fn set_models(models: Vec<crate::ai_client::ModelInfo>) -> Result<()> {
        let path = Self::get_cache_path()?;
        let now = chrono::Utc::now().timestamp();
        
        let cached = CachedModels {
            models,
            timestamp: now,
        };
        
        let content = serde_json::to_string_pretty(&cached)?;
        fs::write(&path, content)?;
        
        Ok(())
    }
    
    #[allow(dead_code)]
    pub fn clear_models() -> Result<()> {
        let path = Self::get_cache_path()?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}
