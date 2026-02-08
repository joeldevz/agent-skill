// src/main.rs (Fragmentos clave)
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::Result;

// --- MODELO DE DATOS (El Manifiesto) ---
#[derive(Serialize, Deserialize, Debug)]
struct SkillManifest {
    version: String,
    skills: HashMap<String, SkillEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SkillEntry {
    url: String,     // URL original del repo
    branch: String,  // main, master, etc.
    local_path: String,
    last_updated: String,
}

// --- COMANDOS CLI ---
#[derive(Parser)]
#[command(name = "skillctl")]
#[command(about = "Gestor de Skills para Agentes de IA", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa un nuevo proyecto de skills
    Init,
    /// Añade una nueva skill al proyecto
    Add { url: String, skill: String },
    /// Re-descarga todas las skills desde sus URLs originales
    Update,
    /// Genera los archivos de configuración para los editores detectados
    Sync {
        #[arg(short, long)]
        editors: Vec<String>, // ej: --editors cursor,antigravity
    },
    /// Lista todas las skills instaladas
    List,
}

// --- LÓGICA DE UPDATE ---
fn update_skills() -> Result<()> {
    let manifest_path = Path::new("skills.toml");
    if !manifest_path.exists() {
        println!("❌ No se encontró skills.toml. Usa 'add' primero.");
        return Ok(());
    }

    let content = fs::read_to_string(manifest_path)?;
    let mut manifest: SkillManifest = toml::from_str(&content)?;

    println!("🔄 Buscando actualizaciones para {} skills...", manifest.skills.len());

    for (name, entry) in &manifest.skills {
        println!("   ⬇️ Actualizando {}...", name);
        // Reutilizamos la lógica de descarga (download_file)
        // Aquí podrías comprobar hashes git si quisieras ser muy preciso,
        // pero re-descargar el RAW file es rápido y efectivo.
        download_skill_file(&entry.url, &entry.branch, name)?;
    }
    
    // Actualizamos timestamp en el toml (opcional)
    fs::write(manifest_path, toml::to_string(&manifest)?)?;
    
    println!("✅ Todas las skills están al día.");
    // Auto-ejecutamos sync para reflejar cambios
    sync_editors(vec!["cursor".to_string(), "antigravity".to_string()])?;
    
    Ok(())
}

// --- LÓGICA DE SYNC MULTI-EDITOR ---
fn sync_editors(targets: Vec<String>) -> Result<()> {
    let manifest = load_manifest()?; // Función helper que lee skills.toml

    for editor in targets {
        match editor.as_str() {
            "cursor" => generate_cursor_config(&manifest)?,
            "antigravity" => generate_antigravity_config(&manifest)?,
            "vscode" => generate_vscode_config(&manifest)?, // Copilot instructions
            _ => println!("⚠️ Editor desconocido: {}", editor),
        }
    }
    Ok(())
}

// Generador para Cursor (.cursorrules)
fn generate_cursor_config(manifest: &SkillManifest) -> Result<()> {
    let mut instructions = String::from("# Rules generadas por Skill-CLI\n\n");
    
    for (name, entry) in &manifest.skills {
        // Opción A: Referencia al archivo (si el editor sabe leer paths)
        instructions.push_str(&format!("## Skill: {}\n", name));
        instructions.push_str(&format!("Reference: .cursor/skills/{}/SKILL.md\n\n", name));
        
        // Opción B (Más robusta): Leer el contenido e inyectarlo si es pequeño
        // let content = fs::read_to_string(&entry.local_path)?;
        // instructions.push_str(&content);
    }

    fs::write(".cursorrules", instructions)?;
    println!("✅ .cursorrules actualizado.");
    Ok(())
}

// Generador para Antigravity (.antigravity)
fn generate_antigravity_config(manifest: &SkillManifest) -> Result<()> {
    // Supongamos que Antigravity usa JSON o un formato diferente
    // O tal vez soporta "Symlinks" virtuales.
    
    let mut config_lines = Vec::new();
    config_lines.push("PROJECT_CONTEXT:".to_string());

    for (name, entry) in &manifest.skills {
        // Imaginemos que Antigravity necesita path absoluto
        let abs_path = fs::canonicalize(&entry.local_path)?;
        config_lines.push(format!("  - IMPORT_SKILL: {}", abs_path.display()));
    }

    fs::write(".antigravity", config_lines.join("\n"))?;
    println!("✅ .antigravity actualizado.");
    Ok(())
}

// Generador para VSCode (.github/copilot-instructions.md)
fn generate_vscode_config(manifest: &SkillManifest) -> Result<()> {
    let mut instructions = String::from("# GitHub Copilot Instructions\n\n");
    
    for (name, entry) in &manifest.skills {
        instructions.push_str(&format!("## Skill: {}\n", name));
        instructions.push_str(&format!("Path: {}\n\n", entry.local_path));
    }

    fs::create_dir_all(".github")?;
    fs::write(".github/copilot-instructions.md", instructions)?;
    println!("✅ .github/copilot-instructions.md actualizado.");
    Ok(())
}

// --- LÓGICA DE INIT ---
fn init_project() -> Result<()> {
    let manifest_path = Path::new("skills.toml");
    if manifest_path.exists() {
        println!("⚠️  El archivo skills.toml ya existe.");
        return Ok(());
    }

    let default_content = r#"# Manifiesto de Skills
version = "1.0"

[skills]
# Las skills se añadirán aquí automáticamente al usar 'add'
"#;

    fs::write(manifest_path, default_content)?;
    
    // Crear carpetas necesarias
    fs::create_dir_all(".cursor/skills")?;
    
    println!("✅ Proyecto inicializado. Se ha creado 'skills.toml'.");
    println!("🚀 Prueba ahora: npx skillctl add <url> --skill <nombre>");
    
    Ok(())
}

// --- LÓGICA DE LIST ---
fn list_skills() -> Result<()> {
    let manifest_path = Path::new("skills.toml");
    if !manifest_path.exists() {
        println!("❌ No se encontró skills.toml. Ejecuta 'skillctl init' primero.");
        return Ok(());
    }

    let content = fs::read_to_string(manifest_path)?;
    let manifest: SkillManifest = toml::from_str(&content)?;

    if manifest.skills.is_empty() {
        println!("📦 No hay skills instaladas.");
        println!("💡 Usa 'skillctl add <url> --skill <nombre>' para añadir una.");
    } else {
        println!("📦 Skills instaladas ({}):", manifest.skills.len());
        for (name, entry) in &manifest.skills {
            println!("  • {} ({})", name, entry.url);
            println!("    └─ Branch: {} | Path: {}", entry.branch, entry.local_path);
        }
    }
    
    Ok(())
}

// --- HELPER: Cargar manifiesto ---
fn load_manifest() -> Result<SkillManifest> {
    let manifest_path = Path::new("skills.toml");
    if !manifest_path.exists() {
        anyhow::bail!("No se encontró skills.toml. Ejecuta 'skillctl init' primero.");
    }
    
    let content = fs::read_to_string(manifest_path)?;
    let manifest: SkillManifest = toml::from_str(&content)?;
    Ok(manifest)
}

// --- HELPER: Descargar archivo de skill ---
fn download_skill_file(url: &str, branch: &str, name: &str) -> Result<()> {
    // Implementación simplificada - aquí iría la lógica real de descarga
    println!("   ⬇️ Descargando {} desde {} (branch: {})", name, url, branch);
    // TODO: Implementar descarga real usando reqwest o similar
    Ok(())
}

// --- FUNCIÓN PRINCIPAL ---
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => init_project()?,
        Commands::Add { url, skill } => {
            println!("🔧 Añadiendo skill '{}' desde {}...", skill, url);
            // Aquí iría la lógica de add_skill(url, skill)
            println!("⚠️  Comando 'add' aún no implementado completamente.");
        }
        Commands::Update => update_skills()?,
        Commands::Sync { editors } => sync_editors(editors.clone())?,
        Commands::List => list_skills()?,
    }

    Ok(())
}