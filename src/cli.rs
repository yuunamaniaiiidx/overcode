use anyhow::{Context, Result};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Index,
    Build,
}

#[derive(Debug)]
pub struct Cli {
    pub command: Command,
    pub root_dir: PathBuf,
}

impl Cli {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        
        if args.len() < 2 {
            anyhow::bail!("Usage: {} <index|build> [directory]", args[0]);
        }

        let command = match args[1].as_str() {
            "index" => Command::Index,
            "build" => Command::Build,
            _ => anyhow::bail!("Unknown command: {}. Use 'index' or 'build'", args[1]),
        };

        let root_dir = if args.len() > 2 {
            PathBuf::from(&args[2])
        } else {
            std::env::current_dir()?
        };

        let root_dir = root_dir
            .canonicalize()
            .with_context(|| format!("Failed to canonicalize path: {:?}", root_dir))?;

        Ok(Self { command, root_dir })
    }
}

