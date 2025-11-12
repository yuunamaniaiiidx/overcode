use crate::cli::{Cli, Command};
use crate::test::process_test;
use crate::run::process_run;

pub fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_default_env().try_init().ok();
    
    let cli = Cli::parse()?;

    match cli.command {
        Command::Init => {
            crate::config::Config::init_config(&cli.root_dir)?;
            crate::podman_install::ensure_podman()?;
            crate::podman_image::ensure_images(&cli.config_path)?;
        }
        Command::Test => {
            crate::config::Config::init_config(&cli.root_dir)?;
            crate::podman_image::ensure_images(&cli.config_path)?;
            process_test(&cli.config_path)?;
        }
        Command::Run => {
            crate::config::Config::init_config(&cli.root_dir)?;
            crate::podman_image::ensure_images(&cli.config_path)?;
            process_run(&cli.config_path, &cli.extra_args)?;
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "overcode/driver/cli/cli.rs"]
mod driver_cli_cli;

#[cfg(test)]
#[path = "overcode/driver/config/config.rs"]
mod driver_config_config;

#[cfg(test)]
#[path = "overcode/driver/podman_image/podman_image.rs"]
mod driver_podman_image_podman_image;

#[cfg(test)]
#[path = "overcode/driver/podman_install/podman_install.rs"]
mod driver_podman_install_podman_install;

#[cfg(test)]
#[path = "overcode/driver/run/run.rs"]
mod driver_run_run;

#[cfg(test)]
#[path = "overcode/driver/test/test.rs"]
mod driver_test_test;