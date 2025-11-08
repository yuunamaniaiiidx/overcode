mod bazel_builder;
mod cli;
mod config;
mod file_hash_index;
mod file_index;
mod hash;
mod index_manager;
mod processor;
mod rust_parser;
mod scanner;
mod storage;

use crate::cli::{Cli, Command};
use crate::index_manager::process_index;
use crate::storage::Storage;
use std::process::Command as ProcessCommand;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse()?;

    match cli.command {
        Command::Init => {
            config::Config::init_config(&cli.root_dir)?;
        }
        Command::Index => {
            config::Config::init_config(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
        }
        Command::Build => {
            // 最新のhistoryファイルを取得
            let storage = Storage::new(&cli.root_dir)?;
            let latest_history = storage.get_latest_history_path()?;
            
            match latest_history {
                Some((_timestamp, history_path)) => {
                    // BUILDファイルとWORKSPACEファイルを生成
                    let (build_file_path, workspace_path) = bazel_builder::generate_build_files(
                        &cli.root_dir,
                        &history_path,
                    )?;
                    
                    println!("Generated BUILD file at: {:?}", build_file_path);
                    println!("Generated WORKSPACE file at: {:?}", workspace_path);
                    
                    // BAZELコマンドを実行（.overcode/buildsディレクトリから）
                    let builds_dir = cli.root_dir.join(".overcode").join("builds");
                    
                    let output = ProcessCommand::new("bazel")
                        .arg("build")
                        .arg("//:sources")
                        .current_dir(&builds_dir)
                        .output()?;
                    
                    // 標準出力と標準エラー出力を表示
                    if !output.stdout.is_empty() {
                        println!("{}", String::from_utf8_lossy(&output.stdout));
                    }
                    if !output.stderr.is_empty() {
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                    }
                    
                    if !output.status.success() {
                        anyhow::bail!("BAZEL build failed with exit code: {:?}", output.status.code());
                    }
                    
                    println!("BAZEL build completed successfully");
                }
                None => {
                    anyhow::bail!("No history file found. Please run 'index' command first.");
                }
            }
        }
    }

    Ok(())
}
