use anyhow::{Context, Result, bail};
use std::process::Command;
use log::info;

/// 指定されたイメージをダウンロード
pub fn pull_image(image: &str) -> Result<()> {
    info!("Pulling image: {}", image);
    
    let status = Command::new("podman")
        .args(&["pull", image])
        .status()
        .with_context(|| format!("Failed to execute podman pull for image: {}", image))?;
    
    if !status.success() {
        bail!("Failed to pull image: {}. Command exited with status: {:?}", image, status.code());
    }
    
    info!("Successfully pulled image: {}", image);
    Ok(())
}

