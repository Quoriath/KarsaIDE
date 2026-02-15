use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: i64,
    pub mode: String, // 'editor' | 'agent'
    pub title: String,
    pub context_path: Option<String>,
    pub model: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(db_path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch("
            PRAGMA journal_mode=WAL;
            PRAGMA synchronous=NORMAL;
            PRAGMA cache_size=10000;
            PRAGMA temp_store=MEMORY;
        ")?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode TEXT NOT NULL,
                title TEXT NOT NULL,
                context_path TEXT,
                model TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
            )",
            [],
        )?;
        
        // Add indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversations_mode ON conversations(mode)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_conversations_updated ON conversations(updated_at DESC)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id)",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp DESC)",
            [],
        )?;
        
        Ok(Self { conn })
    }
    
    fn get_db_path() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?
            .join("karsa-ide");
        
        std::fs::create_dir_all(&data_dir)?;
        Ok(data_dir.join("chat_history.db"))
    }
    
    pub fn create_conversation(&self, mode: &str, title: &str, context_path: Option<&str>, model: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        
        self.conn.execute(
            "INSERT INTO conversations (mode, title, context_path, model, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![mode, title, context_path, model, now, now],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn get_conversations(&self, mode: Option<&str>, limit: i64) -> Result<Vec<Conversation>> {
        let query = if mode.is_some() {
            "SELECT id, mode, title, context_path, model, created_at, updated_at
             FROM conversations WHERE mode = ?1
             ORDER BY updated_at DESC LIMIT ?2"
        } else {
            "SELECT id, mode, title, context_path, model, created_at, updated_at
             FROM conversations
             ORDER BY updated_at DESC LIMIT ?1"
        };
        
        let mut stmt = self.conn.prepare(query)?;
        
        let rows = if let Some(m) = mode {
            stmt.query_map(params![m, limit], Self::map_conversation)?
        } else {
            stmt.query_map(params![limit], Self::map_conversation)?
        };
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    
    fn map_conversation(row: &rusqlite::Row) -> rusqlite::Result<Conversation> {
        Ok(Conversation {
            id: row.get(0)?,
            mode: row.get(1)?,
            title: row.get(2)?,
            context_path: row.get(3)?,
            model: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
    
    pub fn add_message(&self, conversation_id: i64, role: &str, content: &str) -> Result<i64> {
        let now = chrono::Utc::now().timestamp();
        
        self.conn.execute(
            "INSERT INTO messages (conversation_id, role, content, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![conversation_id, role, content, now],
        )?;
        
        // Update conversation updated_at
        self.conn.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id],
        )?;
        
        Ok(self.conn.last_insert_rowid())
    }
    
    pub fn get_messages(&self, conversation_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, conversation_id, role, content, timestamp
             FROM messages WHERE conversation_id = ?1
             ORDER BY timestamp ASC"
        )?;
        
        let rows = stmt.query_map(params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
    
    pub fn delete_conversation(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
        Ok(())
    }
    
    // Batch operations for efficiency
    pub fn add_messages_batch(&self, conversation_id: i64, messages: Vec<(&str, &str)>) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        let now = chrono::Utc::now().timestamp();
        
        for (role, content) in messages {
            tx.execute(
                "INSERT INTO messages (conversation_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
                params![conversation_id, role, content, now],
            )?;
        }
        
        tx.execute(
            "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
            params![now, conversation_id],
        )?;
        
        tx.commit()?;
        Ok(())
    }
    
    pub fn search_conversations(&self, query: &str, mode: Option<&str>) -> Result<Vec<Conversation>> {
        let sql = if mode.is_some() {
            "SELECT id, mode, title, context_path, model, created_at, updated_at
             FROM conversations WHERE mode = ?1 AND title LIKE ?2
             ORDER BY updated_at DESC LIMIT 50"
        } else {
            "SELECT id, mode, title, context_path, model, created_at, updated_at
             FROM conversations WHERE title LIKE ?1
             ORDER BY updated_at DESC LIMIT 50"
        };
        
        let mut stmt = self.conn.prepare_cached(sql)?;
        let search_pattern = format!("%{}%", query);
        
        let rows = if let Some(m) = mode {
            stmt.query_map(params![m, search_pattern], Self::map_conversation)?
        } else {
            stmt.query_map(params![search_pattern], Self::map_conversation)?
        };
        
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }
}
