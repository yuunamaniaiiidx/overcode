mod cli;
mod config;
mod current_dir;
mod file_hash_index;
mod file_index;
mod hash;
mod index_manager;
mod podman;
mod processor;
mod rust_parser;
mod scanner;
mod storage;

use crate::cli::{Cli, Command};
use crate::index_manager::process_index;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse()?;

    match cli.command {
        Command::Init => {
            // initコマンドの場合は設定ファイルの初期化とPodmanの初期化を実行
            config::Config::init_config(&cli.root_dir)?;
            podman::init_podman()?;
        }
        Command::Index => {
            process_index(&cli.root_dir)?;
        }
    }

    Ok(())
}
