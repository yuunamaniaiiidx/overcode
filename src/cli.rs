use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Init,
    Test,
    Run,
}

#[derive(Debug)]
pub struct Cli {
    pub command: Command,
    pub root_dir: PathBuf,
    pub config_path: PathBuf,
    pub extra_args: Vec<String>,
}

fn find_config_dir(config_path: &Path) -> Result<PathBuf> {
    let config_path = config_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", config_path))?;

    if config_path.is_dir() {
        anyhow::bail!(
            "Config file path must be a file, not a directory: {:?}",
            config_path
        );
    }

    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found: {:?}",
            config_path
        );
    }

    Ok(config_path)
}

fn find_config_in_current_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    let config_path = current_dir.join("overcode.toml");

    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found. Please create 'overcode.toml' in the current directory ({:?}) or specify it with --config option.",
            current_dir
        );
    }

    Ok(config_path)
}

impl Cli {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        
        if args.len() < 2 {
            anyhow::bail!("Usage: {} <command> [--config <config_file>] [-- extra_args...]\n  For 'run' command, you can pass additional arguments after '--'", args[0]);
        }

        let command = match args[1].as_str() {
            "init" => Command::Init,
            "test" => Command::Test,
            "run" => Command::Run,
            _ => anyhow::bail!("Unknown command: {}. Use 'init', 'test', or 'run'", args[1]),
        };

        let (args_for_config, extra_args) = if matches!(command, Command::Run) {
            let dash_dash_pos = args.iter().position(|arg| arg == "--");
            
            let args_before_dash_dash = if let Some(pos) = dash_dash_pos {
                &args[..pos]
            } else {
                &args[..]
            };
            
            let extra = if let Some(pos) = dash_dash_pos {
                args[pos + 1..].to_vec()
            } else {
                Vec::new()
            };
            
            (args_before_dash_dash, extra)
        } else {
            (&args[..], Vec::new())
        };

        let config_path = if let Some(config_pos) = args_for_config.iter().position(|arg| arg == "--config") {
            if config_pos + 1 >= args_for_config.len() {
                anyhow::bail!("--config option requires a file path");
            }
            let config_file = &args_for_config[config_pos + 1];
            let config_path = PathBuf::from(config_file);
            if matches!(command, Command::Init) {
                config_path
            } else {
                find_config_dir(&config_path)?
            }
        } else {
            if matches!(command, Command::Init) {
                let current_dir = std::env::current_dir()
                    .context("Failed to get current directory")?;
                current_dir.join("overcode.toml")
            } else {
                find_config_in_current_dir()?
            }
        };

        let root_dir = config_path
            .parent()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| anyhow::anyhow!("Config file has no parent directory"))?;

        Ok(Self { command, root_dir, config_path, extra_args })
    }
}

