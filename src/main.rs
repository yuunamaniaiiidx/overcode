mod build;
mod cli;
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

    println!("Scanning directory: {:?}", cli.root_dir);

    // ディレクトリをスキャン
    let files = scanner::scan_directory(&cli.root_dir)?;
    println!("Found {} files to process", files.len());

    // .overcodeディレクトリの準備
    let storage = Storage::new(&cli.root_dir)?;

    match cli.command {
        Command::Index => {
            process_index(&storage, &files, &cli.root_dir)?;
        }
        Command::Build => {
            let file_index = process_index(&storage, &files, &cli.root_dir)?;
            process_build(&file_index)?;
        }
    }

    Ok(())
}
