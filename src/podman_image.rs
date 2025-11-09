use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;
use log::{info, warn};
use crate::config;

/// 指定されたイメージが既に存在するか確認
fn image_exists(image: &str) -> bool {
    let output = Command::new("podman")
        .args(&["image", "exists", image])
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

/// 指定されたイメージをダウンロード
fn pull_image(image: &str) -> Result<()> {
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

/// 設定ファイルからイメージリストを読み込み、存在しないものはダウンロードする
pub fn ensure_images(root_dir: &Path) -> Result<()> {
    let config = config::Config::load_from_root(root_dir)?;
    
    if config.images.is_empty() {
        info!("No images specified in config");
        return Ok(());
    }
    
    info!("Checking {} image(s)...", config.images.len());
    
    for image_entry in &config.images {
        let image_name = &image_entry.name;
        if image_exists(image_name) {
            info!("Image already exists: {}", image_name);
        } else {
            warn!("Image not found: {}, pulling...", image_name);
            pull_image(image_name)?;
        }
    }
    
    info!("All images are available");
    Ok(())
}

