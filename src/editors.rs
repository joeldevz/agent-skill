use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use strum_macros::{EnumIter, Display};

#[derive(Debug, Clone, Serialize, Deserialize, EnumIter, Display, PartialEq, Eq, Hash)]
pub enum EditorType {
    Cursor,
    Windsurf,
    Antigravity,
    ClaudeCode,
    Cline,
    Roo,
    OpenHands,
    Trae,
    #[serde(rename = "GitHub Copilot")]
    Copilot,
    Continue,
    VSCode,
}

impl EditorType {
    pub fn config_file(&self) -> PathBuf {
        match self {
            EditorType::Cursor => PathBuf::from(".cursorrules"),
            EditorType::Windsurf => PathBuf::from(".windsurfrules"),
            EditorType::Antigravity => PathBuf::from(".agent/rules.md"),
            EditorType::ClaudeCode => PathBuf::from(".claude/config"),
            EditorType::Cline => PathBuf::from(".cline/config"),
            EditorType::Roo => PathBuf::from(".roo/config"),
            EditorType::OpenHands => PathBuf::from(".openhands/config"),
            EditorType::Trae => PathBuf::from(".trae/config"),
            EditorType::Copilot => PathBuf::from(".github/copilot-instructions.md"),
            EditorType::Continue => PathBuf::from(".continue/config.json"),
            EditorType::VSCode => PathBuf::from(".vscode/settings.json"),
        }
    }

    pub fn skills_dir(&self) -> PathBuf {
        match self {
            EditorType::Cursor => PathBuf::from(".cursor/skills"),
            EditorType::Windsurf => PathBuf::from(".windsurf/skills"),
            EditorType::Antigravity => PathBuf::from(".agent/skills"),
            EditorType::ClaudeCode => PathBuf::from(".claude/skills"),
            EditorType::Cline => PathBuf::from(".cline/skills"),
            EditorType::Roo => PathBuf::from(".roo/skills"),
            EditorType::OpenHands => PathBuf::from(".openhands/skills"),
            EditorType::Trae => PathBuf::from(".trae/skills"),
            EditorType::Copilot => PathBuf::from(".github/skills"),
            EditorType::Continue => PathBuf::from(".continue/skills"),
            EditorType::VSCode => PathBuf::from(".vscode/skills"),
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        match self {
            EditorType::Cursor => PathBuf::from(".cursor"),
            EditorType::Windsurf => PathBuf::from(".windsurf"),
            EditorType::Antigravity => PathBuf::from(".agent"),
            EditorType::ClaudeCode => PathBuf::from(".claude"),
            EditorType::Cline => PathBuf::from(".cline"),
            EditorType::Roo => PathBuf::from(".roo"),
            EditorType::OpenHands => PathBuf::from(".openhands"),
            EditorType::Trae => PathBuf::from(".trae"),
            EditorType::Copilot => PathBuf::from(".github"),
            EditorType::Continue => PathBuf::from(".continue"),
            EditorType::VSCode => PathBuf::from(".vscode"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillConfig {
    pub active_editors: Vec<EditorType>,
    pub store_path: String,
    pub skills: HashMap<String, SkillEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub url: String,
    pub local_path: String,
    pub hash: String,
    pub last_updated: String,
}

pub fn default_store_path() -> String {
    ".skillctl/store".to_string()
}

pub fn load_config() -> Result<SkillConfig> {
    let content = fs::read_to_string("skills.json")
        .context("Configuration file not found. Please run 'skillctl init' first.")?;
    
    let config: SkillConfig = serde_json::from_str(&content)
        .context("Failed to parse skills.json. The file may be corrupted.")?;
    
    Ok(config)
}

pub fn save_config(config: &SkillConfig) -> Result<()> {
    let json = serde_json::to_string_pretty(config)
        .context("Failed to serialize configuration")?;
    
    fs::write("skills.json", json)
        .context("Failed to write skills.json")?;
    
    Ok(())
}

/// Auto-detect installed editors by checking for their config directories
pub fn detect_installed_editors() -> Vec<EditorType> {
    use strum::IntoEnumIterator;
    
    let mut detected = Vec::new();
    
    for editor in EditorType::iter() {
        let config_dir = editor.config_dir();
        if config_dir.exists() {
            detected.push(editor);
        }
    }
    
    detected
}

/// Inject a skill reference into an editor's configuration
pub fn inject_reference(editor: &EditorType, skill_name: &str, skill_path: &Path) -> Result<()> {
    let relative_path = skill_path.to_string_lossy();
    
    // CASO ESPECIAL: Cursor usa .cursor/rules/*.mdc
    if let EditorType::Cursor = editor {
        let rules_dir = Path::new(".cursor/rules");
        fs::create_dir_all(rules_dir)
            .context("Failed to create .cursor/rules directory")?;

        let rule_file = rules_dir.join(format!("{}.mdc", skill_name));
        let content = format!(
            "---\ndescription: Skill {}\nglobs: *\n---\n# {}\n\nRead logic from: {}\n",
            skill_name,
            skill_name,
            relative_path
        );
        fs::write(&rule_file, content)
            .context("Failed to write Cursor rule file")?;
        return Ok(());
    }

    let config_file = editor.config_file();
    let current_content = if config_file.exists() { 
        fs::read_to_string(&config_file)
            .context("Failed to read editor config file")? 
    } else { 
        String::new() 
    };
    
    // Lógica específica por editor para inyección en archivo único
    let injection = match editor {
        EditorType::Antigravity => format!("\n### Skill: {}\nRefer to logic in: `{}`\n", skill_name, relative_path),
        EditorType::Cline | EditorType::Roo => format!("\nRunning context for {}: See {}\n", skill_name, relative_path),
        _ => format!("\n- Skill ({}) -> Read file: {}\n", skill_name, relative_path),
    };

    if !current_content.contains(&format!("Skill: {}", skill_name)) && !current_content.contains(&format!("Skill ({})", skill_name)) {
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create editor config directory")?;
        }
        let mut file = fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config_file)
            .context("Failed to open editor config file")?;
        
        use std::io::Write;
        write!(file, "{}", injection)
            .context("Failed to write to editor config file")?;
    }
    Ok(())
}

/// Remove a skill reference from an editor's configuration
pub fn remove_reference(editor: &EditorType, skill_name: &str) -> Result<()> {
    // CASO ESPECIAL: Cursor usa .cursor/rules/*.mdc
    if let EditorType::Cursor = editor {
        let rule_file = Path::new(".cursor/rules").join(format!("{}.mdc", skill_name));
        if rule_file.exists() {
            fs::remove_file(rule_file)
                .context("Failed to remove Cursor rule file")?;
        }
        return Ok(());
    }

    let config_file = editor.config_file();
    if !config_file.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&config_file)
        .context("Failed to read editor config file")?;
    
    // Remove lines that reference this skill
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_next = false;

    for line in lines {
        // Skip lines that mention the skill
        if line.contains(&format!("Skill: {}", skill_name)) 
            || line.contains(&format!("Skill ({})", skill_name))
            || line.contains(&format!("context for {}", skill_name)) {
            skip_next = true;
            continue;
        }
        
        // Skip the next line if it was a reference path
        if skip_next && (line.contains("Read file:") || line.contains("Refer to logic") || line.contains("See ")) {
            skip_next = false;
            continue;
        }
        
        skip_next = false;
        new_lines.push(line);
    }

    fs::write(&config_file, new_lines.join("\n"))
        .context("Failed to write updated editor config file")?;
    
    Ok(())
}

/// Inject or update memory context in an editor's configuration
pub fn inject_memory_context(editor: &EditorType, memory_content: &str) -> Result<()> {
    // CASO ESPECIAL: Cursor usa .cursor/rules/memory.mdc
    if let EditorType::Cursor = editor {
        let rules_dir = Path::new(".cursor/rules");
        fs::create_dir_all(rules_dir)
            .context("Failed to create .cursor/rules directory")?;

        let rule_file = rules_dir.join("memory.mdc");
        let content = format!(
            "---\ndescription: Global Active Memory\nglobs: *\n---\n{}",
            memory_content
        );
        fs::write(&rule_file, content)
            .context("Failed to write Cursor memory file")?;
        return Ok(());
    }

    // CASO ESPECIAL: Antigravity usa .agent/memory.md
    if let EditorType::Antigravity = editor {
        let agent_dir = Path::new(".agent");
        if !agent_dir.exists() {
             fs::create_dir_all(agent_dir)
                .context("Failed to create .agent directory")?;
        }
        let memory_file = agent_dir.join("memory.md");
        fs::write(&memory_file, memory_content)
            .context("Failed to write Antigravity memory file")?;
        return Ok(());
    }

    let config_file = editor.config_file();
    
    // If config file doesn't exist, create it with memory content if it has content
    if !config_file.exists() {
        if memory_content.trim().is_empty() {
            return Ok(());
        }
        
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create editor config directory")?;
        }
        fs::write(&config_file, memory_content)
            .context("Failed to write editor config file")?;
        return Ok(())
    }

    let current_content = fs::read_to_string(&config_file)
        .context("Failed to read editor config file")?;
    
    // Check if memory context already exists
    let header = "# 🧠 Active Memory Context";
    
    let new_content = if current_content.contains(header) {
        // Replace existing memory block
        let parts: Vec<&str> = current_content.split(header).collect();
        // Keep everything before the header, and append new memory content
        // Note: This simplistic approach assumes memory block is attached at the end or we replace from header onwards
        // To be safer, we assume memory block is always at the end for non-Cursor editors
        format!("{}{}", parts[0].trim_end(), memory_content)
    } else {
        // Append new memory block
        format!("{}\n\n{}", current_content.trim_end(), memory_content)
    };

    fs::write(&config_file, new_content)
        .context("Failed to update editor config with memory")?;
    
    Ok(())
}
