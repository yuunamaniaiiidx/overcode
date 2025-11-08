mod build;
mod cli;
mod config;
mod current_dir;
mod file_hash_index;
mod file_index;
mod hash;
mod index_manager;
mod processor;
mod rust_parser;
mod scanner;
mod storage;

use cli::{Cli, Command};
use build::process_build;
use index_manager::process_index;
use storage::Storage;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse()?;

    println!("Root directory: {:?}", cli.root_dir);

    // 設定ファイルを読み込む
    let config = config::Config::load_from_root(&cli.root_dir)?;
    let ignore_patterns = config.get_ignore_patterns();
    if !ignore_patterns.is_empty() {
        println!("Loaded {} ignore pattern(s)", ignore_patterns.len());
    }

    println!("Scanning directory: {:?}", cli.root_dir);

    // ディレクトリをスキャン
    let files = scanner::scan_directory(&cli.root_dir, &ignore_patterns)?;
    println!("Found {} files to process", files.len());

    // .overcodeディレクトリの準備
    let storage = Storage::new(&cli.root_dir)?;

    match cli.command {
        Command::Index => {
            process_index(&storage, &files, &cli.root_dir)?;
        }
        Command::Build => {
            process_index(&storage, &files, &cli.root_dir)?;
            process_build(&storage)?;
        }
    }

    Ok(())
}
