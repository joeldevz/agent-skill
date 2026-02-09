use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, Display};
use cliclack::{intro, outro, log, spinner, select, multiselect, confirm, outro_note};
use console::style;
use sha2::{Sha256, Digest};
use chrono::Utc;
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

// --- 1. CONFIGURACIÓN DE EDITORES (Portado de agents.ts) ---
#[derive(Debug, Serialize, Deserialize, EnumIter, Display, Clone, PartialEq, Eq)]
enum EditorType {
    #[strum(serialize = "Cursor")]
    Cursor,
    #[strum(serialize = "Windsurf")]
    Windsurf,
    #[strum(serialize = "Antigravity")]
    Antigravity,
    #[strum(serialize = "VS Code")]
    VSCode,
    #[strum(serialize = "Claude Code")]
    ClaudeCode,
    #[strum(serialize = "Cline")]
    Cline,
    #[strum(serialize = "Roo Code")]
    Roo,
    #[strum(serialize = "OpenHands")]
    OpenHands,
    #[strum(serialize = "Trae")]
    Trae,
    #[strum(serialize = "GitHub Copilot")]
    Copilot,
    #[strum(serialize = "Continue")]
    Continue,
}

impl EditorType {
    // Portado de la propiedad `skillsDir` de agents.ts
    fn skills_dir(&self) -> PathBuf {
        match self {
            EditorType::Cursor => PathBuf::from(".cursor/skills"),
            EditorType::Windsurf => PathBuf::from(".windsurf/skills"),
            EditorType::Antigravity => PathBuf::from(".agent/skills"), // Universal Agent dir
            EditorType::ClaudeCode => PathBuf::from(".claude/skills"),
            EditorType::Cline => PathBuf::from(".cline/skills"),
            EditorType::Roo => PathBuf::from(".roo/skills"),
            EditorType::OpenHands => PathBuf::from(".openhands/skills"),
            EditorType::Trae => PathBuf::from(".trae/skills"),
            EditorType::Copilot => PathBuf::from(".copilot/skills"),
            EditorType::Continue => PathBuf::from(".continue/skills"),
            EditorType::VSCode => PathBuf::from(".vscode/skills"),
        }
    }

    // Archivo donde inyectamos la referencia ("Linker")
    fn config_file(&self) -> PathBuf {
        match self {
            EditorType::Cursor => PathBuf::from(".cursorrules"),
            EditorType::Windsurf => PathBuf::from(".windsurfrules"),
            EditorType::Antigravity => PathBuf::from(".agent/rules/rules.md"), 
            EditorType::ClaudeCode => PathBuf::from(".claude/config.md"), // Hipotético
            EditorType::Cline => PathBuf::from(".clinerules"),
            EditorType::Roo => PathBuf::from(".clinerules"), // Roo usa formato Cline a menudo
            EditorType::OpenHands => PathBuf::from(".openhands/memory.md"),
            EditorType::Trae => PathBuf::from(".trae/config.md"),
            EditorType::Copilot => PathBuf::from(".github/copilot-instructions.md"),
            EditorType::Continue => PathBuf::from(".continue/config.json"), // JSON injection is harder, skipping logic for brevity
            EditorType::VSCode => PathBuf::from(".vscode/skills.md"),
        }
    }
}

// --- 2. ESTRUCTURAS DE DATOS ---

#[derive(Serialize, Deserialize, Debug)]
struct SkillConfig {
    #[serde(default)]
    active_editors: Vec<EditorType>,
    #[serde(default = "default_store_path")]
    store_path: String,
    skills: HashMap<String, SkillEntry>,
}

fn default_store_path() -> String {
    ".skillctl/store".to_string()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SkillEntry {
    url: String,
    #[serde(rename = "relative_path")] // Map JSON's relative_path to local_path
    local_path: String,
    hash: String,
    last_updated: String,
    #[serde(default)]
    installed_in: Vec<EditorType>, 
}

#[derive(Deserialize, Debug)]
struct RegistryItem {
    name: String,
    description: String,
    url: String,
    #[serde(default)]
    skill_path: Option<String>,
}

// --- 3. CLI ARGUMENTS ---
#[derive(Parser)]
#[command(name = "skillctl", version = "0.0.7", about = "Secure AI Skill Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    /// Adds a skill from URL. Usage: add <url> --skill <name>
    Add { url: String, #[arg(long)] skill: String },
    /// Restore skills from skills.json
    Install,
    /// Search the community registry
    /// Search the community registry
    Search,
    /// List installed skills
    List,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Intro minimalista Astro-style
    // Intro 'Astro-like' - Badge + Version + Friendly Face inside logic
    let banner = format!(
        "{} {} {}",
        style(" skillctl ").on_cyan().black(),
        style(format!("v{}", env!("CARGO_PKG_VERSION"))).cyan(),
        style("Launch sequence initiated.").dim()
    );
    intro(banner)?;

    // Optional: "Houston" style greeting
    log::info(format!("{}  {}", style("◠ ◡ ◠").cyan(), "Time to build intelligent agents."))?;

    match &cli.command {
        Commands::Init => init_project()?,
        Commands::Add { url, skill } => add_skill_logic(url, skill)?,
        Commands::Install => install_all()?,
        Commands::Search => search_skills()?,
        Commands::List => list_skills()?,
    }

    Ok(())
}

// --- LÓGICA DE INIT ---
fn init_project() -> Result<()> {
    if Path::new("skills.json").exists() {
        log::warning("skills.json already exists.")?;
        let overwrite = confirm("Do you want to re-initialize? (This will overwrite skills.json)").interact()?;
        if !overwrite {
            outro("Skipping init.")?;
            return Ok(());
        }
    }

    log::info("Initializing secure skill environment.")?;

    let editors: Vec<EditorType> = EditorType::iter().collect();
    // Convert to config format for multiselect
    let items: Vec<(EditorType, String, String)> = editors.iter()
        .map(|e| (e.clone(), e.to_string(), format!("Uses {}", e.skills_dir().display())))
        .collect();

    // Use multiselect to allow multiple editors
    let selected_editors: Vec<EditorType> = cliclack::multiselect("Which AI Editors are you using?")
        .items(&items)
        .interact()?;

    if selected_editors.is_empty() {
        outro("No editors selected. Exiting.")?;
        return Ok(());
    }

    let config = SkillConfig {
        active_editors: selected_editors.clone(),
        store_path: default_store_path(),
        skills: HashMap::new(),
    };

    let spin = spinner();
    spin.start("Scaffolding directories...");
    
    save_config(&config)?;
    
    // Create central store
    fs::create_dir_all(&config.store_path)?;

    // Setup each editor
    for editor in &selected_editors {
        // We don't necessarily need skills_dir for each editor if we are using a central store,
        // BUT we might need it if we want to symlink/copy. 
        // For now, let's just create the config file folder.
        let rules_file = editor.config_file();
        if let Some(parent) = rules_file.parent() { fs::create_dir_all(parent)?; }
        
        if !rules_file.exists() {
            fs::write(&rules_file, format!("# AI Rules for {}\n", editor))?;
        }
        
        // Ensure "skills" dir exists if we rely on it, though we are moving to store strategy.
        // Let's create it just in case users want manual placement too.
        fs::create_dir_all(editor.skills_dir())?;
    }

    spin.stop("Environment ready.");
    
    outro_note(
        style("Setup Complete").cyan(),
        format!("Configured for {:?}. Try: npx skillctl search", selected_editors)
    )?;
    Ok(())
}

// --- LÓGICA CORE: ADD & INTEGRITY CHECK ---
fn add_skill_logic(repo_url: &str, skill_name: &str) -> Result<()> {
    let mut config = load_config().context("Please run 'init' first.")?;
    
    let spin = spinner();
    spin.start(format!("Fetching {}...", skill_name));

    // 1. Transformar URL a Raw (GitHub support)
    let raw_base = repo_url.replace("github.com", "raw.githubusercontent.com").trim_end_matches('/').to_string();
    let target_url = format!("{}/main/skills/{}/SKILL.md", raw_base, skill_name); // Ajustar según estructura real

    // 2. Descargar contenido en memoria
    let resp = reqwest::blocking::get(&target_url)?;
    if !resp.status().is_success() {
        spin.stop("Failed");
        log::error(format!("404 Not Found: {}", target_url))?;
        return Ok(());
    }
    let content = resp.text()?;

    // 3. CALCULAR HASH (Integrity)
    let new_hash = calculate_hash(&content);
    spin.stop("Downloaded.");

    // 4. Verificar contra lo instalado
    if let Some(existing_entry) = config.skills.get(skill_name) {
        if existing_entry.hash != new_hash {
            log::warning("⚠️  Integrity Check: Content differs from installed version.")?;
            let should_update = confirm("Do you want to overwrite local skill with remote version?").interact()?;
            if !should_update {
                outro("Update cancelled.")?;
                return Ok(());
            }
        } else {
            log::info("Skill is up to date (Hash match).")?;
        }
    }

    // 5. Guardar archivo en Central Store
    let spin_write = spinner();
    spin_write.start("Installing securely to store...");
    
    let filename = "SKILL.md";
    let store_dir = Path::new(&config.store_path).join(skill_name);
    let local_path = store_dir.join(filename);
    
    fs::create_dir_all(&store_dir)?;
    fs::write(&local_path, &content)?;

    // 6. Actualizar Configuración
    config.skills.insert(skill_name.to_string(), SkillEntry {
        url: repo_url.to_string(),
        local_path: local_path.to_string_lossy().to_string(),
        hash: new_hash,
        last_updated: Utc::now().to_rfc3339(),
        installed_in: config.active_editors.clone(),
    });
    save_config(&config)?;

    // 7. Linkear para CADA editor activo
    for editor in &config.active_editors {
        inject_reference(editor, skill_name, &local_path)?;
    }

    spin_write.stop("Installed.");
    outro(format!("{} is now active for {:?}", style(skill_name).green(), config.active_editors))?;

    Ok(())
}

// --- LÓGICA SEARCH ---
fn search_skills() -> Result<()> {
    let spin = spinner();
    spin.start("Fetching registry...");
    
    // CAMBIA ESTO POR TU URL REAL DE GITHUB RAW
    let registry_url = "https://raw.githubusercontent.com/joeldevz/agent-skills/main/registry.json"; 
    
    let resp = reqwest::blocking::get(registry_url);
    spin.stop("Registry loaded.");

    let items: Vec<RegistryItem> = match resp {
        Ok(r) => r.json().unwrap_or_default(),
        Err(_) => {
            log::error("Could not reach registry. Check your internet.")?;
            return Ok(());
        }
    };

    if items.is_empty() {
        log::warning("Registry is empty.")?;
        return Ok(());
    }

    // Fuzzy Search con Dialoguer (Mejor experiencia que cliclack para esto)
    let options: Vec<String> = items.iter()
        .map(|i| format!("{} - {}", style(&i.name).bold().cyan(), i.description))
        .collect();

    let selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Search skills:")
        .default(0)
        .items(&options)
        .interact_opt()?;

    if let Some(index) = selection {
        let chosen = &items[index];
        let skill_id = chosen.skill_path.as_deref().unwrap_or(&chosen.name);
        
        // Llamada recursiva a la lógica de add
        add_skill_logic(&chosen.url, skill_id)?;
    } else {
        outro("Cancelled.")?;
    }
    Ok(())
}

// --- COMANDO LIST ---
fn list_skills() -> Result<()> {
    let config = load_config().context("Configuration not found. Please run 'skillctl init' first.")?;
    
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

// --- COMANDO INSTALL (RESTORE) ---
// --- COMANDO INSTALL (RESTORE) ---
// --- COMANDO INSTALL (RESTORE) ---
fn install_all() -> Result<()> {
    let config = load_config().context("Configuration not found. Please run 'skillctl init' first.")?;
    log::info(format!("Verifying {} skills...", config.skills.len()))?;

    for (name, entry) in config.skills {
        let local_path = Path::new(&entry.local_path);

        // Si el archivo no existe, intentamos descargarlo de nuevo (Restore)
        if !local_path.exists() {
            let spin = spinner();
            spin.start(format!("Restoring {}...", name));

            // Reconstruir URL (Lógica simplificada de GitHub Raw)
            let raw_base = entry.url.replace("github.com", "raw.githubusercontent.com").trim_end_matches('/').to_string();
            let target_url = format!("{}/main/skills/{}/SKILL.md", raw_base, name);

            match reqwest::blocking::get(&target_url) {
                Ok(resp) if resp.status().is_success() => {
                     let content = resp.text()?;
                     // Verificación de integridad básica
                     let current_hash = calculate_hash(&content);
                     if current_hash != entry.hash {
                         log::warning(format!("⚠️ Hash mismatch for {}. Content changed upstream.", name))?;
                     }

                     if let Some(parent) = local_path.parent() {
                         fs::create_dir_all(parent)?;
                     }
                     fs::write(local_path, content)?;
                     spin.stop("Restored.");
                },
                _ => {
                    spin.stop("Failed.");
                    log::error(format!("Could not restore {} from {}", name, target_url))?;
                }
            }
        }

        // Always check references for all active editors
        for editor in &config.active_editors {
            inject_reference(editor, &name, local_path)?;
        }
    }
    outro("All skills verified and linked.")?;
    Ok(())
}

// --- HELPERS ---

fn calculate_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex::encode(hasher.finalize())
}

fn inject_reference(editor: &EditorType, skill_name: &str, skill_path: &Path) -> Result<()> {
    let relative_path = skill_path.to_string_lossy();
    
    // CASO ESPECIAL: Cursor usa ahora .cursor/rules/*.mdc
    if let EditorType::Cursor = editor {
        let rules_dir = Path::new(".cursor/rules");
        fs::create_dir_all(rules_dir)?;

        let rule_file = rules_dir.join(format!("{}.mdc", skill_name));
        let content = format!(
            "---\ndescription: Skill {}\nglobs: *\n---\n# {}\n\nRead logic from: {}\n",
            skill_name,
            skill_name,
            relative_path
        );
        fs::write(&rule_file, content)?;
        return Ok(());
    }

    let config_file = editor.config_file();
    let current_content = if config_file.exists() { fs::read_to_string(&config_file)? } else { String::new() };
    
    // Lógica específica por editor para inyección en archivo único
    let injection = match editor {
        EditorType::Antigravity => format!("\n### Skill: {}\nRefer to logic in: `{}`\n", skill_name, relative_path),
        EditorType::Cline | EditorType::Roo => format!("\nRunning context for {}: See {}\n", skill_name, relative_path),
        _ => format!("\n- Skill ({}) -> Read file: {}\n", skill_name, relative_path),
    };

    if !current_content.contains(&format!("Skill: {}", skill_name)) && !current_content.contains(&format!("Skill ({})", skill_name)) {
        if let Some(parent) = config_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = fs::OpenOptions::new().append(true).create(true).open(&config_file)?;
        use std::io::Write;
        write!(file, "{}", injection)?;
    }
    Ok(())
}

fn load_config() -> Result<SkillConfig> {
    let content = fs::read_to_string("skills.json")?;
    Ok(serde_json::from_str(&content)?)
}

fn save_config(config: &SkillConfig) -> Result<()> {
    fs::write("skills.json", serde_json::to_string_pretty(config)?)?;
    Ok(())
}