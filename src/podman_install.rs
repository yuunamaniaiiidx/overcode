use anyhow::{Context, Result, bail};
use std::process::Command;
use std::fs;
use log::{info, warn};

fn check_podman_installed() -> bool {
    let output = Command::new("podman")
        .arg("--version")
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

fn detect_os() -> Result<OsType> {
    let os_release_path = "/etc/os-release";
    
    if !std::path::Path::new(os_release_path).exists() {
        bail!("Cannot detect OS: /etc/os-release not found");
    }
    
    let content = fs::read_to_string(os_release_path)
        .with_context(|| format!("Failed to read {}", os_release_path))?;
    
    let mut id: Option<String> = None;
    let mut id_like: Option<String> = None;
    
    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            
            if key == "ID" {
                id = Some(value.to_string());
            } else if key == "ID_LIKE" {
                id_like = Some(value.to_string());
            }
        }
    }
    
    if let Some(ref os_id) = id {
        match os_id.as_str() {
            "ubuntu" | "debian" => return Ok(OsType::Debian),
            "fedora" | "centos" | "rhel" => return Ok(OsType::RedHat),
            _ => {}
        }
    }
    
    if let Some(ref like) = id_like {
        if like.contains("debian") || like.contains("ubuntu") {
            return Ok(OsType::Debian);
        }
        if like.contains("fedora") || like.contains("rhel") || like.contains("centos") {
            return Ok(OsType::RedHat);
        }
    }
    
    warn!("Unknown OS type, defaulting to Debian-based. ID: {:?}, ID_LIKE: {:?}", id, id_like);
    Ok(OsType::Debian)
}

#[derive(Debug, Clone, Copy)]
enum OsType {
    Debian,
    RedHat,
}

fn install_podman(os_type: OsType) -> Result<()> {
    let (cmd, args) = match os_type {
        OsType::Debian => {
            ("apt-get", vec!["install", "-y", "podman"])
        }
        OsType::RedHat => {
            let dnf_available = Command::new("which")
                .arg("dnf")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            
            if dnf_available {
                ("dnf", vec!["install", "-y", "podman"])
            } else {
                ("yum", vec!["install", "-y", "podman"])
            }
        }
    };
    
    info!("Installing podman using: sudo {} {}", cmd, args.join(" "));
    
    let status = Command::new("sudo")
        .arg(cmd)
        .args(&args)
        .status()
        .with_context(|| format!("Failed to execute sudo {} install", cmd))?;
    
    if !status.success() {
        bail!("Failed to install podman. Command exited with status: {:?}", status.code());
    }
    
    info!("podman installed successfully");
    Ok(())
}

pub fn ensure_podman() -> Result<()> {
    if check_podman_installed() {
        info!("podman is already installed");
        return Ok(());
    }
    
    info!("podman is not installed. Detecting OS...");
    let os_type = detect_os()?;
    info!("Detected OS type: {:?}", os_type);
    
    install_podman(os_type)?;
    
    if !check_podman_installed() {
        bail!("podman installation completed but verification failed");
    }
    
    info!("podman installation verified successfully");
    Ok(())
}

