use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "skillctl", version = "0.0.9", about = "Secure AI Skill Manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize skillctl in the current directory
    Init,
    
    /// Add a skill from a repository
    Add { 
        /// Repository URL (e.g., https://github.com/user/repo)
        url: String,
        
        /// Skill name to install
        #[arg(long)] 
        skill: Option<String>,
        
        /// Custom path to SKILL.md within the repository
        #[arg(long)] 
        path: Option<String>,
        
        /// List available skills without installing
        #[arg(long, short = 'l')] 
        list: bool,
    },
    
    /// Remove installed skills
    Remove {
        /// Names of skills to remove
        #[arg(required = true)]
        skills: Vec<String>,
    },
    
    /// Restore skills from skills.json
    Install,
    
    /// Search the community registry
    Search,
    
    /// List installed skills
    List,
    
    /// Manage Active Memory
    #[command(subcommand)]
    Memory(MemoryCommands),
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Add a new memory
    Learn {
        /// The knowledge to remember
        text: String,
    },
    
    /// Remove a memory by ID
    Forget {
        /// Memory ID
        id: String,
    },
    
    /// List all memories
    List,
    
    /// Search memories
    Search {
        /// Query string
        query: String,
    },
}
