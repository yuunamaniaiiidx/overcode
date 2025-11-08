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
mod test;

use crate::cli::{Cli, Command};
use crate::index_manager::process_index;

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
        Command::Test { args } => {
            config::Config::init_config(&cli.root_dir)?;
            test::run_test_command(&cli.root_dir, &args)?;
        }
    }

    Ok(())
}
