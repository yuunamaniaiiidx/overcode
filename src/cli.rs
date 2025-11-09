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
    pub extra_args: Vec<String>,
}

/// 設定ファイル（overcode.toml）から基準ディレクトリを取得する
fn find_config_dir(config_path: &Path) -> Result<PathBuf> {
    let config_path = config_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", config_path))?;

    // ディレクトリの場合はエラー
    if config_path.is_dir() {
        anyhow::bail!(
            "The second argument must be a config file path (overcode.toml), not a directory: {:?}",
            config_path
        );
    }

    // ファイルが存在するか確認
    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found: {:?}",
            config_path
        );
    }

    // ファイル名がovercode.tomlか確認
    if config_path.file_name() != Some(std::ffi::OsStr::new("overcode.toml")) {
        anyhow::bail!(
            "Config file must be named 'overcode.toml', got: {:?}",
            config_path.file_name()
        );
    }

    // 親ディレクトリを基準ディレクトリとして返す
    config_path
        .parent()
        .map(|p| p.to_path_buf())
        .ok_or_else(|| anyhow::anyhow!("Config file has no parent directory"))
}

/// 実行ディレクトリ直下のovercode.tomlを探す
fn find_config_in_current_dir() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()
        .context("Failed to get current directory")?;
    let config_path = current_dir.join("overcode.toml");

    if !config_path.exists() {
        anyhow::bail!(
            "Config file not found. Please create 'overcode.toml' in the current directory ({:?}) or specify it as the second argument.",
            current_dir
        );
    }

    Ok(current_dir)
}

impl Cli {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();
        
        if args.len() < 2 {
            anyhow::bail!("Usage: {} <command> [config_file] [-- extra_args...]\n  For 'run' command, you can pass additional arguments after '--'", args[0]);
        }

        let command = match args[1].as_str() {
            "init" => Command::Init,
            "test" => Command::Test,
            "run" => Command::Run,
            _ => anyhow::bail!("Unknown command: {}. Use 'init', 'test', or 'run'", args[1]),
        };

        // runコマンドの場合のみ、--の後の引数を取得
        let (args_for_config, extra_args) = if matches!(command, Command::Run) {
            // -- の位置を探す
            let dash_dash_pos = args.iter().position(|arg| arg == "--");
            
            // -- より前の引数で設定ファイルを探す
            let args_before_dash_dash = if let Some(pos) = dash_dash_pos {
                &args[..pos]
            } else {
                &args[..]
            };
            
            // -- より後の引数を取得（extra_args）
            let extra = if let Some(pos) = dash_dash_pos {
                args[pos + 1..].to_vec()
            } else {
                Vec::new()
            };
            
            (args_before_dash_dash, extra)
        } else {
            (&args[..], Vec::new())
        };

        // initコマンドの場合は設定ファイルを必須にしない
        let root_dir = if matches!(command, Command::Init) {
            // initコマンドの場合は現在のディレクトリを使用（設定ファイルは不要）
            std::env::current_dir()
                .context("Failed to get current directory")?
        } else if args_for_config.len() > 2 {
            // 第二引数が指定されている場合：設定ファイルのパスとして扱う
            let config_path = PathBuf::from(&args_for_config[2]);
            find_config_dir(&config_path)?
        } else {
            // 第二引数がない場合：実行ディレクトリ直下のovercode.tomlを探す
            find_config_in_current_dir()?
        };

        Ok(Self { command, root_dir, extra_args })
    }
}

