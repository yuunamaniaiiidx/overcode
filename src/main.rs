mod build;
mod cli;
mod config;
mod file_hash_index;
mod file_index;
mod hash;
mod index_manager;
mod podman_image;
mod podman_install;
mod processor;
mod rust_parser;
mod run;
mod scanner;
mod storage;
mod test;

use crate::cli::{Cli, Command};
use crate::index_manager::process_index;
use crate::build::process_build;
use crate::test::process_test;
use crate::run::process_run;

fn main() -> anyhow::Result<()> {
    // 環境変数RUST_LOGが設定されている場合のみログを有効化
    env_logger::Builder::from_default_env().try_init().ok();
    
    let cli = Cli::parse()?;

    match cli.command {
        Command::Init => {
            config::Config::init_config(&cli.root_dir)?;
            podman_install::ensure_podman()?;
            podman_image::ensure_images(&cli.root_dir)?;
        }
        Command::Index => {
            config::Config::init_config(&cli.root_dir)?;
            podman_install::ensure_podman()?;
            podman_image::ensure_images(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
        }
        Command::Build => {
            config::Config::init_config(&cli.root_dir)?;
            podman_image::ensure_images(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
            process_build(&cli.root_dir)?;
        }
        Command::Test => {
            config::Config::init_config(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
            process_build(&cli.root_dir)?;
            process_test(&cli.root_dir)?;
        }
        Command::Run => {
            config::Config::init_config(&cli.root_dir)?;
            podman_image::ensure_images(&cli.root_dir)?;
            process_index(&cli.root_dir)?;
            process_build(&cli.root_dir)?;
            process_run(&cli.root_dir, &cli.extra_args)?;
        }
    }

    Ok(())
}
