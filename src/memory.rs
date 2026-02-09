use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context, bail};
use chrono::{Utc, DateTime};
use uuid::Uuid;
use strum_macros::{EnumString, Display};

#[derive(Debug, Serialize, Deserialize, Clone, EnumString, Display, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub enum MemoryTag {
    Preference,
    Stack,
    Correction,
    Constraint,
    Style,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tag: MemoryTag,
    pub priority: u8,
    pub source: String,
    pub created_at: DateTime<Utc>,
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

    pub fn add_memory(&mut self, content: String, source: String, tag: MemoryTag, priority: u8) -> Result<String> {
        if content.trim().is_empty() {
            bail!("Memory content cannot be empty");
        }

        let id = Uuid::new_v4().to_string()[..8].to_string(); // Short ID
        
        let entry = MemoryEntry {
            id: id.clone(),
            content,
            source,
            tag,
            priority,
            created_at: Utc::now(),
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

    pub fn list_memories(&mut self) -> &Vec<MemoryEntry> {
        // Sort by priority (descending) then by date (descending)
        self.memories.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        &self.memories
    }
    
    pub fn search_memories(&self, query: &str) -> Vec<&MemoryEntry> {
        let query = query.to_lowercase();
        let mut results: Vec<&MemoryEntry> = self.memories.iter()
            .filter(|m| m.content.to_lowercase().contains(&query) || m.tag.to_string().to_lowercase().contains(&query))
            .collect();
        
        // Sort search results by priority as well
        results.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| b.created_at.cmp(&a.created_at))
        });
        
        results
    }
    
    /// Format memories for injection into AI context
    pub fn to_context_string(&mut self) -> String {
        let mut output = String::from("\n# 🧠 Active Memory Context\n\n");
        
        let memories = self.list_memories(); // Uses the sorted list
        
        if memories.is_empty() {
            output.push_str("No memories stored yet.\n");
        } else {
            for memory in memories {
                output.push_str(&format!("- [ID: {}] [{}] (Prio: {}) {}\n", 
                    memory.id, 
                    memory.tag, 
                    memory.priority, 
                    memory.content
                ));
            }
        }
        
        output.push_str("\n# 🛠️ Memory Tools\n");
        output.push_str("- Save: `skillctl memory learn \"text\" --tag <tag> --priority <1-10>`\n");
        output.push_str("- Delete: `skillctl memory forget ID`\n");
        
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_memory_crud_and_sorting() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = MemoryStore::new(temp_dir.path()).unwrap();
        
        // Add low priority
        let id1 = store.add_memory("Low priority".to_string(), "cli".to_string(), MemoryTag::Preference, 1).unwrap();
        
        // Add high priority
        let id2 = store.add_memory("High priority".to_string(), "cli".to_string(), MemoryTag::Constraint, 10).unwrap();
        
        // Add medium priority
        let id3 = store.add_memory("Medium priority".to_string(), "cli".to_string(), MemoryTag::Style, 5).unwrap();
        
        let memories = store.list_memories();
        assert_eq!(memories.len(), 3);
        
        // Check sorting (High, Medium, Low)
        assert_eq!(memories[0].id, id2);
        assert_eq!(memories[1].id, id3);
        assert_eq!(memories[2].id, id1);
        
        // Context String
        let context = store.to_context_string();
        assert!(context.contains("High priority"));
        assert!(context.contains("Prio: 10"));
        
        // Remove
        let removed = store.remove_memory(&id2).unwrap();
        assert!(removed);
        assert_eq!(store.list_memories().len(), 2);
    }
}
