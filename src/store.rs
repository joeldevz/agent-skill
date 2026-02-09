use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use chrono::Utc;
use crate::security::{validate_skill_name, validate_path_in_store};
use crate::editors::{SkillEntry, SkillConfig, save_config};

pub struct SkillStore {
    base_path: PathBuf,
}

impl SkillStore {
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Create store directory if it doesn't exist
        fs::create_dir_all(&base_path)
            .context("Failed to create skill store directory")?;

        Ok(Self { base_path })
    }

    /// Calculate SHA256 hash of content
    pub fn calculate_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    /// Install a skill to the store
    pub fn install_skill(
        &self,
        skill_name: &str,
        content: &str,
        source_url: &str,
    ) -> Result<SkillEntry> {
        // Validate skill name (security check)
        validate_skill_name(skill_name)?;

        // Calculate hash
        let hash = Self::calculate_hash(content);

        // Create skill directory
        let skill_dir = self.base_path.join(skill_name);
        
        // Validate path is within store (security check)
        validate_path_in_store(&self.base_path, &skill_dir)?;

        fs::create_dir_all(&skill_dir)
            .context("Failed to create skill directory")?;

        // Write SKILL.md file
        let skill_file = skill_dir.join("SKILL.md");
        fs::write(&skill_file, content)
            .context("Failed to write SKILL.md file")?;

        // Create entry
        let entry = SkillEntry {
            url: source_url.to_string(),
            local_path: skill_file.to_string_lossy().to_string(),
            hash,
            last_updated: Utc::now().to_rfc3339(),
        };

        Ok(entry)
    }

    /// Check if a skill exists and verify its integrity
    pub fn verify_skill(&self, skill_name: &str, expected_hash: &str) -> Result<bool> {
        validate_skill_name(skill_name)?;

        let skill_file = self.base_path.join(skill_name).join("SKILL.md");
        
        if !skill_file.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(&skill_file)
            .context("Failed to read skill file")?;

        let actual_hash = Self::calculate_hash(&content);
        
        Ok(actual_hash == expected_hash)
    }

    /// Remove a skill from the store
    pub fn remove_skill(&self, skill_name: &str) -> Result<()> {
        validate_skill_name(skill_name)?;

        let skill_dir = self.base_path.join(skill_name);
        
        // Validate path is within store (security check)
        validate_path_in_store(&self.base_path, &skill_dir)?;

        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)
                .context("Failed to remove skill directory")?;
        }

        Ok(())
    }

    /// Get the path to a skill file
    pub fn get_skill_path(&self, skill_name: &str) -> Result<PathBuf> {
        validate_skill_name(skill_name)?;

        let skill_file = self.base_path.join(skill_name).join("SKILL.md");
        
        // Validate path is within store (security check)
        validate_path_in_store(&self.base_path, &skill_file)?;

        Ok(skill_file)
    }

    /// List all installed skills
    pub fn list_skills(&self) -> Result<Vec<String>> {
        let mut skills = Vec::new();

        if !self.base_path.exists() {
            return Ok(skills);
        }

        for entry in fs::read_dir(&self.base_path)
            .context("Failed to read store directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name() {
                    if let Some(name_str) = name.to_str() {
                        // Validate it's a valid skill name
                        if validate_skill_name(name_str).is_ok() {
                            skills.push(name_str.to_string());
                        }
                    }
                }
            }
        }

        Ok(skills)
    }
}

/// Update a skill in the configuration and store
pub fn update_skill_in_config(
    config: &mut SkillConfig,
    skill_name: &str,
    entry: SkillEntry,
) -> Result<()> {
    config.skills.insert(skill_name.to_string(), entry);
    save_config(config)?;
    Ok(())
}

/// Remove a skill from the configuration
pub fn remove_skill_from_config(
    config: &mut SkillConfig,
    skill_name: &str,
) -> Result<Option<SkillEntry>> {
    let entry = config.skills.remove(skill_name);
    save_config(config)?;
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_calculate_hash() {
        let content = "test content";
        let hash = SkillStore::calculate_hash(content);
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_store_creation() {
        let temp_dir = TempDir::new().unwrap();
        let store = SkillStore::new(temp_dir.path());
        assert!(store.is_ok());
    }

    #[test]
    fn test_install_skill() {
        let temp_dir = TempDir::new().unwrap();
        let store = SkillStore::new(temp_dir.path()).unwrap();
        
        let result = store.install_skill(
            "test-skill",
            "# Test Skill\n\nThis is a test.",
            "https://github.com/test/repo"
        );
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_skill_name() {
        let temp_dir = TempDir::new().unwrap();
        let store = SkillStore::new(temp_dir.path()).unwrap();
        
        let result = store.install_skill(
            "../etc/passwd",
            "malicious content",
            "https://evil.com"
        );
        
        assert!(result.is_err());
    }
}
