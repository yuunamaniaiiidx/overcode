use std::path::Path;
use std::process::Command;
use std::collections::HashSet;
use log::{info, warn};
use crate::config;
use crate::podman_image_download;
use anyhow::Result;

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

/// 設定ファイルからイメージリストを読み込み、存在しないものはダウンロードする
pub fn ensure_images(root_dir: &Path) -> Result<()> {
    let config = config::Config::load_from_root(root_dir)?;
    
    // command.test.imageとcommand.run.imageからイメージを収集
    let mut images = HashSet::new();
    
    if let Some(command) = &config.command {
        if let Some(test_config) = &command.test {
            if let Some(image) = &test_config.image {
                images.insert(image.clone());
            }
        }
        if let Some(run_config) = &command.run {
            if let Some(image) = &run_config.image {
                images.insert(image.clone());
            }
        }
    }
    
    if images.is_empty() {
        info!("No images specified in command.test or command.run");
        return Ok(());
    }
    
    info!("Checking {} image(s)...", images.len());
    
    for image_name in &images {
        if image_exists(image_name) {
            info!("Image already exists: {}", image_name);
        } else {
            warn!("Image not found: {}, pulling...", image_name);
            podman_image_download::pull_image(image_name)?;
        }
    }
    
    info!("All images are available");
    Ok(())
}

#[cfg(test)]
#[path = "podman_image/driver/config/config.rs"]
mod driver_config_config;

#[cfg(test)]
#[path = "podman_image/driver/podman_image_download/fail.rs"]
mod driver_podman_image_download_fail;

#[cfg(test)]
#[path = "podman_image/driver/podman_image_download/success.rs"]
mod driver_podman_image_download_success;