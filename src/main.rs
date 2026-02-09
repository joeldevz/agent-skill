mod cli;
mod security;
mod editors;
mod network;
mod store;
mod memory;

use anyhow::{Result, Context};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::fs;
use cliclack::{intro, outro, log, spinner, confirm, outro_note};
use console::style;

use cli::{Cli, Commands, MemoryCommands};
use editors::{EditorType, SkillConfig, default_store_path, load_config, save_config, detect_installed_editors, inject_reference, remove_reference, inject_memory_context};
use network::SecureHttpClient;
use store::{SkillStore, update_skill_in_config, remove_skill_from_config};
use security::validate_skill_name;
use memory::MemoryStore;

fn main() -> Result<()> {
    let cli = Cli::parse();

    intro(format!(
        "{} {} {}",
        style("skillctl").cyan().bold(),
        style(env!("CARGO_PKG_VERSION")).dim(),
        "Launch sequence initiated."
    ))?;

    log::step(format!("{} Time to build intelligent agents.", 
        style("◠ ◡ ◠").cyan()))?;

    match &cli.command {
        Commands::Init => cmd_init()?,
        Commands::Add { url, skill, path, list } => {
            if *list {
                cmd_list_available(url, path.clone())?;
            } else if let Some(skill_name) = skill {
                cmd_add(url, skill_name, path.clone())?;
            } else {
                log::error("--skill <name> is required when not using --list")?;
            }
        },
        Commands::Remove { skills } => cmd_remove(skills)?,
        Commands::Install => cmd_install()?,
        Commands::Search => cmd_search()?,
        Commands::List => cmd_list()?,
        Commands::Memory(subcommand) => cmd_memory(subcommand)?,
    }

    Ok(())
}

// ============================================================================
// COMMAND: INIT
// ============================================================================

fn cmd_init() -> Result<()> {
    if Path::new("skills.json").exists() {
        log::warning("skills.json already exists.")?;
        let overwrite = confirm("Do you want to re-initialize? (This will overwrite skills.json)").interact()?;
        if !overwrite {
            outro("Skipping init.")?;
            return Ok(());
        }
    }

    log::info("Initializing secure skill environment.")?;

    // Auto-detect installed editors
    let detected_editors = detect_installed_editors();
    
    let selected_editors = if !detected_editors.is_empty() {
        log::info(format!("Detected {} editor(s): {:?}", 
            detected_editors.len(), 
            detected_editors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        ))?;
        
        let use_detected = confirm("Use detected editors?").interact()?;
        if use_detected {
            detected_editors
        } else {
            prompt_editor_selection()?
        }
    } else {
        prompt_editor_selection()?
    };

    if selected_editors.is_empty() {
        outro("No editors selected. Exiting.")?;
        return Ok(());
    }

    let config = SkillConfig {
        active_editors: selected_editors.clone(),
        store_path: default_store_path(),
        skills: std::collections::HashMap::new(),
    };

    let spin = spinner();
    spin.start("Scaffolding directories...");
    
    save_config(&config)?;
    fs::create_dir_all(&config.store_path)?;

    for editor in &selected_editors {
        let rules_file = editor.config_file();
        if let Some(parent) = rules_file.parent() { 
            fs::create_dir_all(parent)?; 
        }
        
        if !rules_file.exists() {
            fs::write(&rules_file, format!("# AI Rules for {}\n", editor))?;
        }
        
        fs::create_dir_all(editor.skills_dir())?;
    }

    spin.stop("Environment ready.");
    
    // Inject memory instructions into all editors
    let memory_store = MemoryStore::new(&config.store_path)?;
    let memory_context = memory_store.to_context_string();
    
    for editor in &selected_editors {
        if let Err(e) = inject_memory_context(editor, &memory_context) {
            log::warning(format!("Could not inject memory context for {}: {}", editor, e))?;
        }
    }
    
    // Install the built-in memory skill from repository
    let spin = spinner();
    spin.start("Installing memory skill...");
    
    // Explicitly use the singular repository name
    let memory_repo = "https://github.com/joeldevz/agent-skill";
    let memory_skill = "memory";
    
    match cmd_add(memory_repo, memory_skill, None) {
        Ok(_) => spin.stop("Memory skill installed."),
        Err(e) => {
            log::warning(format!("Memory skill auto-install skipped: {}", e))?;
            spin.stop("Memory skill installation skipped.");
        }
    }
    
    outro_note(
        style("Setup Complete").cyan(),
        format!("Configured for {:?}. Try: skillctl search", selected_editors)
    )?;
    
    Ok(())
}

fn prompt_editor_selection() -> Result<Vec<EditorType>> {
    use strum::IntoEnumIterator;
    
    let editors: Vec<EditorType> = EditorType::iter().collect();
    let items: Vec<(EditorType, String, String)> = editors.iter()
        .map(|e| (e.clone(), e.to_string(), format!("Uses {}", e.skills_dir().display())))
        .collect();

    let selected_editors: Vec<EditorType> = cliclack::multiselect("Which AI Editors are you using?")
        .items(&items)
        .interact()?;

    Ok(selected_editors)
}

// ============================================================================
// COMMAND: ADD
// ============================================================================

fn cmd_add(repo_url: &str, skill_name: &str, custom_path: Option<String>) -> Result<()> {
    // Validate skill name (security)
    validate_skill_name(skill_name)
        .context("Invalid skill name")?;

    let mut config = load_config()
        .context("Please run 'skillctl init' first.")?;
    
    let spin = spinner();
    spin.start(format!("Fetching {}...", skill_name));

    // Create secure HTTP client
    let client = SecureHttpClient::new()?;

    // Try to find and download the skill
    let (content, _path) = client.find_skill(repo_url, skill_name, custom_path)
        .context("Failed to download skill")?;

    spin.stop("Downloaded.");

    // Check if skill already exists and verify hash
    if let Some(existing) = config.skills.get(skill_name) {
        let new_hash = SkillStore::calculate_hash(&content);
        
        if new_hash != existing.hash {
            log::warning("Skill exists with different content.")?;
            let should_update = confirm("Do you want to overwrite local skill with remote version?").interact()?;
            if !should_update {
                outro("Update cancelled.")?;
                return Ok(());
            }
        } else {
            log::info("Skill is up to date (Hash match).")?;
        }
    }

    // Install to store
    let store = SkillStore::new(&config.store_path)?;
    let entry = store.install_skill(skill_name, &content, repo_url)?;
    
    // Update config
    update_skill_in_config(&mut config, skill_name, entry.clone())?;

    // Inject references for all active editors
    let skill_path = store.get_skill_path(skill_name)?;
    for editor in &config.active_editors {
        inject_reference(editor, skill_name, &skill_path)?;
    }

    log::success("Installed.")?;
    outro(format!("{} is now active for {:?}", skill_name, config.active_editors))?;

    Ok(())
}

// ============================================================================
// COMMAND: REMOVE
// ============================================================================

fn cmd_remove(skill_names: &[String]) -> Result<()> {
    let mut config = load_config()
        .context("Configuration not found. Please run 'skillctl init' first.")?;
    
    if config.skills.is_empty() {
        log::warning("No skills installed.")?;
        return Ok(());
    }

    log::info(format!("Removing {} skill(s)...", skill_names.len()))?;
    
    let store = SkillStore::new(&config.store_path)?;
    let mut removed_count = 0;
    let mut not_found = Vec::new();

    for skill_name in skill_names {
        // Validate skill name (security)
        if let Err(e) = validate_skill_name(skill_name) {
            log::warning(format!("Invalid skill name '{}': {}", skill_name, e))?;
            continue;
        }

        if remove_skill_from_config(&mut config, skill_name)?.is_some() {
            // Remove from filesystem
            store.remove_skill(skill_name)?;

            // Remove references from all active editors
            for editor in &config.active_editors {
                remove_reference(editor, skill_name)?;
            }

            removed_count += 1;
            log::info(format!("✓ Removed {}", style(skill_name).green()))?;
        } else {
            not_found.push(skill_name.clone());
        }
    }

    if removed_count > 0 {
        outro(format!("Removed {} skill(s)", removed_count))?;
    }
    
    if !not_found.is_empty() {
        log::warning(format!("Skills not found: {}", not_found.join(", ")))?;
    }

    Ok(())
}

// ============================================================================
// COMMAND: LIST
// ============================================================================

fn cmd_list() -> Result<()> {
    let config = load_config()
        .context("Configuration not found. Please run 'skillctl init' first.")?;
    
    if config.skills.is_empty() {
        log::warning("No skills installed.")?;
        outro_note(style("Hint").cyan(), "Try running 'skillctl search' to find skills.")?;
        return Ok(());
    }

    log::info(format!("{} installed skills:", style(config.skills.len()).cyan()))?;

    for (name, entry) in config.skills {
        let date = chrono::DateTime::parse_from_rfc3339(&entry.last_updated)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| "??".to_string());
            
        println!("   {} {}  {}", 
            style("●").green(), 
            style(&name).bold(), 
            style(format!("[{}]", date)).dim()
        );
    }
    println!();
    Ok(())
}

// ============================================================================
// COMMAND: INSTALL (Restore from skills.json)
// ============================================================================

fn cmd_install() -> Result<()> {
    let config = load_config()
        .context("Configuration not found. Please run 'skillctl init' first.")?;
    
    if config.skills.is_empty() {
        log::warning("No skills in configuration.")?;
        return Ok(());
    }

    log::info(format!("Restoring {} skill(s)...", config.skills.len()))?;

    let store = SkillStore::new(&config.store_path)?;
    let client = SecureHttpClient::new()?;

    for (name, entry) in &config.skills {
        // Validate skill name (security)
        if let Err(e) = validate_skill_name(name) {
            log::warning(format!("Skipping invalid skill name '{}': {}", name, e))?;
            continue;
        }

        let local_path = store.get_skill_path(name)?;

        // Check if file exists and verify integrity
        if !store.verify_skill(name, &entry.hash)? {
            let spin = spinner();
            spin.start(format!("Restoring {}...", name));

            // Re-download
            match client.download(&entry.url) {
                Ok(content) => {
                    fs::create_dir_all(local_path.parent().unwrap())?;
                    fs::write(&local_path, content)?;
                    spin.stop("Restored.");
                },
                Err(e) => {
                    spin.stop("Failed.");
                    log::error(format!("Could not restore {}: {}", name, e))?;
                }
            }
        }

        // Always check references for all active editors
        for editor in &config.active_editors {
            inject_reference(editor, name, &local_path)?;
        }
    }

    outro("All skills verified and linked.")?;
    Ok(())
}

// ============================================================================
// COMMAND: SEARCH
// ============================================================================

fn cmd_search() -> Result<()> {
    let spin = spinner();
    spin.start("Fetching registry...");
    
    let registry_url = "https://raw.githubusercontent.com/joeldevz/agent-skill/refs/heads/main/registry.json";
    
    let client = SecureHttpClient::new()?;
    let content = client.download(registry_url)?;
    
    spin.stop("Registry loaded.");

    #[derive(serde::Deserialize)]
    struct RegistryItem {
        name: String,
        description: String,
        url: String,
        #[serde(default)]
        skill_path: Option<String>,
    }

    let items: Vec<RegistryItem> = serde_json::from_str(&content)
        .unwrap_or_default();

    if items.is_empty() {
        log::warning("Registry is empty.")?;
        return Ok(());
    }

    // Fuzzy Search
    let options: Vec<String> = items.iter()
        .map(|i| format!("{} - {}", style(&i.name).bold().cyan(), i.description))
        .collect();

    let selection = dialoguer::FuzzySelect::new()
        .with_prompt("Search skills")
        .items(&options)
        .interact_opt()?;

    if let Some(index) = selection {
        let chosen = &items[index];
        let skill_id = chosen.skill_path.as_deref().unwrap_or(&chosen.name);
        
        cmd_add(&chosen.url, skill_id, None)?;
    } else {
        outro("Cancelled.")?;
    }

    Ok(())
}

// ============================================================================
// COMMAND: LIST AVAILABLE
// ============================================================================

fn cmd_list_available(_repo_url: &str, _custom_path: Option<String>) -> Result<()> {
    let spin = spinner();
    spin.start("Discovering available skills...");
    spin.stop("Discovery complete.");
    
    log::warning("Skill discovery from remote repos is limited without cloning.")?;
    log::info("Try installing a specific skill with: skillctl add <url> --skill <name>")?;
    outro("For full discovery, the repository would need to be cloned locally")?;
    
    Ok(())
}
// ============================================================================
// COMMAND: MEMORY
// ============================================================================

fn cmd_memory(command: &MemoryCommands) -> Result<()> {
    let config = load_config()
        .context("Configuration not found. Please run 'skillctl init' first.")?;
    
    // Initialize or load memory store
    let mut memory_store = MemoryStore::new(&config.store_path)?;

    match command {
        MemoryCommands::Learn { text } => {
            log::info("Learning new memory...")?;
            let id = memory_store.add_memory(text.clone(), "user-cli".to_string())?;
            log::success(format!("Memory learned! [ID: {}]", id))?;
        },
        MemoryCommands::Forget { id } => {
            log::info(format!("Forgetting memory {}...", id))?;
            if memory_store.remove_memory(id)? {
                log::success("Memory forgotten.")?;
            } else {
                log::warning(format!("Memory ID {} not found.", id))?;
                return Ok(());
            }
        },
        MemoryCommands::List => {
            let memories = memory_store.list_memories();
            if memories.is_empty() {
                log::info("No memories found.")?;
            } else {
                println!("\n🧠 Active Memories:");
                for m in memories {
                    println!("   • [{}] {}", style(&m.id).cyan(), m.content);
                }
                println!();
            }
            return Ok(());
        },
        MemoryCommands::Search { query } => {
            let results = memory_store.search_memories(query);
            if results.is_empty() {
                log::info("No matching memories found.")?;
            } else {
                println!("\n🔍 Search Results:");
                for m in results {
                    println!("   • [{}] {}", style(&m.id).cyan(), m.content);
                }
                println!();
            }
            return Ok(());
        }
    }

    // Sync changes to editors
    if matches!(command, MemoryCommands::Learn { .. } | MemoryCommands::Forget { .. }) {
        let context = memory_store.to_context_string();
        
        let spin = spinner();
        spin.start("Syncing to editors...");
        
        for editor in &config.active_editors {
            if let Err(e) = inject_memory_context(editor, &context) {
                log::error(format!("Failed to sync memory to {}: {}", editor, e))?;
            }
        }
        
        spin.stop("Synced.");
    }

    Ok(())
}
