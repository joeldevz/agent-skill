use clap::{Parser, Subcommand};
use colored::*;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use dialoguer::{Select, theme::ColorfulTheme};
use std::collections::HashMap;

// --- MODELO DE DATOS (JSON) ---
#[derive(Serialize, Deserialize, Debug)]
struct SkillConfig {
    editor: String, // "cursor", "antigravity", "vscode"
    skills: HashMap<String, SkillEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SkillEntry {
    url: String,
    local_path: String,
}

impl Default for SkillConfig {
    fn default() -> Self {
        Self {
            editor: "cursor".to_string(),
            skills: HashMap::new(),
        }
    }
}

// --- CLI ARGUMENTS ---
#[derive(Parser)]
#[command(name = "skillctl")]
#[command(version = "1.0.0")]
#[command(about = "Gestor de Skills tipo Vercel", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa el proyecto y elige editor
    Init,
    /// Añade una skill. Uso: skillctl add <URL> --skill <NOMBRE>
    Add {
        url: String,
        /// Nombre de la skill a extraer
        #[arg(long)] // Esto hace que sea --skill <nombre>
        skill: String,
    },
    /// Instala todas las skills definidas en skills.json
    Install,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => init_project()?,
        Commands::Add { url, skill } => add_skill(url, skill)?,
        Commands::Install => install_all()?,
    }
    Ok(())
}

// --- COMANDO: INIT (Interactivo) ---
fn init_project() -> Result<()> {
    let config_path = Path::new("skills.json");
    if config_path.exists() {
        println!("⚠️  Ya existe 'skills.json'.");
        return Ok(());
    }

    println!("{}", "🚀 Inicializando Skill Controller...".bold().cyan());

    // Menú interactivo
    let editors = vec!["Cursor (.cursor/skills)", "Antigravity (.antigravity)", "VSCode (.vscode)"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("¿Qué editor vas a usar?")
        .default(0)
        .items(&editors)
        .interact()
        .unwrap();

    let editor_key = match selection {
        0 => "cursor",
        1 => "antigravity",
        _ => "vscode",
    };

    let config = SkillConfig {
        editor: editor_key.to_string(),
        skills: HashMap::new(),
    };

    save_config(&config)?;
    
    // Crear carpeta base según editor
    let base_dir = get_skills_dir(editor_key);
    fs::create_dir_all(&base_dir)?;

    println!("✅ Configuración guardada en 'skills.json'. Editor: {}", editor_key.green());
    Ok(())
}

// --- COMANDO: ADD ---
fn add_skill(repo_url: &str, skill_name: &str) -> Result<()> {
    // 1. Cargar config para saber dónde guardar
    let mut config = load_config()?;
    let skills_dir = get_skills_dir(&config.editor);

    println!("{} {}...", "📦 Añadiendo skill:".blue(), skill_name);

    // 2. Lógica de descarga (GitHub Raw)
    let raw_base = repo_url
        .replace("github.com", "raw.githubusercontent.com")
        .trim_end_matches('/')
        .to_string();
    
    // URL: .../main/skills/{nombre}/SKILL.md (Ajustar según estructura real del repo)
    let target_url = format!("{}/main/skills/{}/SKILL.md", raw_base, skill_name);
    
    // 3. Descargar
    let content = download_file(&target_url)?;

    // 4. Guardar archivo
    let skill_folder = skills_dir.join(skill_name);
    fs::create_dir_all(&skill_folder)?;
    let file_path = skill_folder.join("SKILL.md");
    fs::write(&file_path, &content)?;

    println!("✅ Skill guardada en: {:?}", file_path);

    // 5. Actualizar JSON
    config.skills.insert(skill_name.to_string(), SkillEntry {
        url: repo_url.to_string(),
        local_path: file_path.to_string_lossy().to_string(),
    });
    save_config(&config)?;

    // 6. Actualizar configuración del editor (Integración)
    update_editor_config(&config.editor, skill_name, &file_path)?;

    Ok(())
}

// --- COMANDO: INSTALL ---
fn install_all() -> Result<()> {
    let config = load_config().context("No se encontró skills.json. Ejecuta 'init' primero.")?;
    
    println!("🔄 Restaurando {} skills para {}...", config.skills.len(), config.editor);

    for (name, entry) in &config.skills {
        // Re-usamos la lógica de add pero sin duplicar entradas en el json
        // (Aquí simplificado: solo descargamos el archivo de nuevo)
        
        let raw_base = entry.url
            .replace("github.com", "raw.githubusercontent.com")
            .trim_end_matches('/')
            .to_string();
        let target_url = format!("{}/main/skills/{}/SKILL.md", raw_base, name);

        match download_file(&target_url) {
            Ok(content) => {
                let path = Path::new(&entry.local_path);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(path, content)?;
                println!(" - ✅ {}", name);
            },
            Err(_) => println!(" - ❌ Error descargando {}", name),
        }
    }
    println!("✨ Instalación completada.");
    Ok(())
}

// --- HELPERS ---

fn get_skills_dir(editor: &str) -> std::path::PathBuf {
    match editor {
        "antigravity" => Path::new(".antigravity/skills").to_path_buf(),
        "vscode" => Path::new(".vscode/skills").to_path_buf(),
        _ => Path::new(".cursor/skills").to_path_buf(), // Default
    }
}

fn load_config() -> Result<SkillConfig> {
    let content = fs::read_to_string("skills.json")?;
    let config: SkillConfig = serde_json::from_str(&content)?;
    Ok(config)
}

fn save_config(config: &SkillConfig) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    fs::write("skills.json", content)?;
    Ok(())
}

fn download_file(url: &str) -> Result<String> {
    let resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        anyhow::bail!("404 Not Found");
    }
    Ok(resp.text()?)
}

fn update_editor_config(editor: &str, skill_name: &str, path: &Path) -> Result<()> {
    // Aquí implementas la lógica específica para inyectar en .cursorrules o .antigravity
    // Ejemplo simple para cursor:
    if editor == "cursor" {
        let rule_file = Path::new(".cursorrules");
        let line = format!("\n# Skill: {}\nReference: {}\n", skill_name, path.display());
        
        // Append
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(rule_file)?;
        use std::io::Write;
        write!(file, "{}", line)?;
    }
    Ok(())
}