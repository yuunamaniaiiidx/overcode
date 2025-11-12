use anyhow::Context;
use std::path::Path;
use std::process::Command;
use std::io::Write;
use crate::config::Config;
use log::info;

fn execute_run_command(
    run_config: &crate::config::RunTestConfig,
    root_dir: &Path,
    extra_args: &[String],
) -> anyhow::Result<()> {
    let root_dir_str = root_dir.display().to_string();
    
    let mut processed_args: Vec<String> = run_config.args
        .iter()
        .map(|arg| {
            arg.replace("{root_dir}", &root_dir_str)
        })
        .collect();
    
    processed_args.extend_from_slice(extra_args);
    
    if let Some(ref image) = run_config.image {
        info!("Executing in podman container (image: {}): {} {:?}", image, run_config.command, processed_args);
        
        let mut podman_args = vec![
            "run".to_string(),
            "--rm".to_string(),
            "-v".to_string(),
            format!("{}:{}", root_dir_str, root_dir_str),
            "-w".to_string(),
            root_dir_str.clone(),
            image.clone(),
            run_config.command.clone(),
        ];
        podman_args.extend(processed_args);
        
        let output = Command::new("podman")
            .args(&podman_args)
            .output()
            .with_context(|| format!("Failed to execute podman run for image: {}", image))?;
        
        std::io::stdout().write_all(&output.stdout)
            .context("Failed to write stdout")?;
        std::io::stderr().write_all(&output.stderr)
            .context("Failed to write stderr")?;
        
        if !output.status.success() {
            anyhow::bail!(
                "Run command failed with exit code: {:?}",
                output.status.code()
            );
        }
    } else {
        info!("Executing: {} {:?} (from {:?})", run_config.command, processed_args, root_dir);
        
        let output = Command::new(&run_config.command)
            .args(&processed_args)
            .current_dir(root_dir)
            .output()
            .with_context(|| format!("Failed to execute command: {}", run_config.command))?;
        
        std::io::stdout().write_all(&output.stdout)
            .context("Failed to write stdout")?;
        std::io::stderr().write_all(&output.stderr)
            .context("Failed to write stderr")?;
        
        if !output.status.success() {
            anyhow::bail!(
                "Run command failed with exit code: {:?}",
                output.status.code()
            );
        }
    }
    
    Ok(())
}

pub fn process_run(config_path: &Path, extra_args: &[String]) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    let root_dir = config_path
        .parent()
            .ok_or_else(|| anyhow::anyhow!("Config file has no parent directory"))?;
    
    let run_config = config.command
        .as_ref()
        .and_then(|c| c.run.as_ref())
        .ok_or_else(|| anyhow::anyhow!("[command.run] section not found in overcode.toml"))?;
    
    info!("Executing run command");
    if !extra_args.is_empty() {
        info!("Additional arguments: {:?}", extra_args);
    }
    
    execute_run_command(run_config, root_dir, extra_args)?;
    
    info!("Run command completed successfully");
    
    Ok(())
}

#[cfg(test)]
#[path = "run/driver/config/config.rs"]
mod driver_config_config;

