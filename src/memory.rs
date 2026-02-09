use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub source: String,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MemoryStore {
    pub memories: Vec<MemoryEntry>,
    #[serde(skip)]
    file_path: PathBuf,
}

impl MemoryStore {
    pub fn new(store_path: impl AsRef<Path>) -> Result<Self> {
        let store_path = store_path.as_ref();
        let file_path = store_path.join("memory.json");
        
        let mut store = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .context("Failed to read memory file")?;
            serde_json::from_str(&content)
                .context("Failed to parse memory file")?
        } else {
            MemoryStore::default()
        };
        
        store.file_path = file_path;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create memory store directory")?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize memory store")?;
        
        fs::write(&self.file_path, json)
            .context("Failed to write memory file")?;
            
        Ok(())
    }

    pub fn add_memory(&mut self, content: String, source: String) -> Result<String> {
        if content.trim().is_empty() {
            bail!("Memory content cannot be empty");
        }

        let id = Uuid::new_v4().to_string()[..8].to_string(); // Short ID
        
        let entry = MemoryEntry {
            id: id.clone(),
            content,
            source,
            timestamp: Utc::now().to_rfc3339(),
        };
        
        self.memories.push(entry);
        self.save()?;
        
        Ok(id)
    }

    pub fn remove_memory(&mut self, id: &str) -> Result<bool> {
        let original_len = self.memories.len();
        self.memories.retain(|m| m.id != id);
        
        if self.memories.len() < original_len {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_memories(&self) -> &Vec<MemoryEntry> {
        &self.memories
    }
    
    pub fn search_memories(&self, query: &str) -> Vec<&MemoryEntry> {
        let query = query.to_lowercase();
        self.memories.iter()
            .filter(|m| m.content.to_lowercase().contains(&query))
            .collect()
    }
    
    /// Format memories for injection into AI context
    pub fn to_context_string(&self) -> String {
        if self.memories.is_empty() {
            return String::new();
        }

        let mut output = String::from("\n# 🧠 Active Memory Context\n\n");
        for memory in &self.memories {
            output.push_str(&format!("- [ID: {}] {}\n", memory.id, memory.content));
        }
        
        output.push_str("\n# 🛠️ Memory Tools\n");
        output.push_str("- Save: `skillctl memory learn \"text\"`\n");
        output.push_str("- Delete: `skillctl memory forget ID`\n");
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_memory_crud() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = MemoryStore::new(temp_dir.path()).unwrap();
        
        // Add
        let id = store.add_memory("User likes Rust".to_string(), "cli".to_string()).unwrap();
        assert!(!id.is_empty());
        assert_eq!(store.list_memories().len(), 1);
        
        // Context String
        let context = store.to_context_string();
        assert!(context.contains("User likes Rust"));
        assert!(context.contains(&id));
        
        // Remove
        let removed = store.remove_memory(&id).unwrap();
        assert!(removed);
        assert_eq!(store.list_memories().len(), 0);
    }
}
