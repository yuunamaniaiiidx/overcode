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
use crate::bazel_builder::{process_build, process_test_target};

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
            config::Config::init_config(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
            process_build(&cli.root_dir)?;
        }
        Command::Test => {
            config::Config::init_config(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
            process_test_target(&cli.root_dir)?;
        }
    }

    Ok(())
}
