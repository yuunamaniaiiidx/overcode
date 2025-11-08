use anyhow::{Context, Result};
use std::process::Command as StdCommand;

/// Podmanがインストールされているか確認する
pub fn is_podman_installed() -> bool {
    StdCommand::new("podman")
        .arg("--version")
        .output()
        .is_ok()
}

/// Podmanのバージョンを取得する
pub fn get_podman_version() -> Result<String> {
    let output = StdCommand::new("podman")
        .arg("--version")
        .output()
        .context("Failed to execute podman --version")?;

    if !output.status.success() {
        anyhow::bail!("podman --version failed with status: {}", output.status);
    }

    let version = String::from_utf8(output.stdout)
        .context("Failed to parse podman version output")?;

    Ok(version.trim().to_string())
}

/// Podmanをインストールする
pub fn install_podman() -> Result<()> {
    println!("Podmanが見つかりません。インストールを開始します...");

    // パッケージマネージャーを検出
    let package_manager = detect_package_manager()?;

    match package_manager {
        PackageManager::Apt => {
            println!("aptを使用してPodmanをインストールします...");
            install_with_apt()?;
        }
        PackageManager::Dnf => {
            println!("dnfを使用してPodmanをインストールします...");
            install_with_dnf()?;
        }
        PackageManager::Yum => {
            println!("yumを使用してPodmanをインストールします...");
            install_with_yum()?;
        }
        PackageManager::Pacman => {
            println!("pacmanを使用してPodmanをインストールします...");
            install_with_pacman()?;
        }
        PackageManager::Zypper => {
            println!("zypperを使用してPodmanをインストールします...");
            install_with_zypper()?;
        }
    }

    // インストール後の確認
    if is_podman_installed() {
        let version = get_podman_version()?;
        println!("Podmanのインストールが完了しました: {}", version);
        Ok(())
    } else {
        anyhow::bail!("Podmanのインストールに失敗しました。手動でインストールしてください。");
    }
}

#[derive(Debug, Clone, Copy)]
enum PackageManager {
    Apt,
    Dnf,
    Yum,
    Pacman,
    Zypper,
}

fn detect_package_manager() -> Result<PackageManager> {
    // 各パッケージマネージャーの存在を確認
    if command_exists("apt-get") {
        return Ok(PackageManager::Apt);
    }
    if command_exists("dnf") {
        return Ok(PackageManager::Dnf);
    }
    if command_exists("yum") {
        return Ok(PackageManager::Yum);
    }
    if command_exists("pacman") {
        return Ok(PackageManager::Pacman);
    }
    if command_exists("zypper") {
        return Ok(PackageManager::Zypper);
    }

    anyhow::bail!("サポートされているパッケージマネージャーが見つかりませんでした。手動でPodmanをインストールしてください。");
}

fn command_exists(cmd: &str) -> bool {
    StdCommand::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn install_with_apt() -> Result<()> {
    // パッケージリストを更新
    let update_status = StdCommand::new("sudo")
        .arg("apt-get")
        .arg("update")
        .status()
        .context("Failed to run apt-get update")?;

    if !update_status.success() {
        anyhow::bail!("apt-get update failed");
    }

    // Podmanをインストール
    let install_status = StdCommand::new("sudo")
        .arg("apt-get")
        .arg("install")
        .arg("-y")
        .arg("podman")
        .status()
        .context("Failed to run apt-get install podman")?;

    if !install_status.success() {
        anyhow::bail!("apt-get install podman failed");
    }

    Ok(())
}

fn install_with_dnf() -> Result<()> {
    let status = StdCommand::new("sudo")
        .arg("dnf")
        .arg("install")
        .arg("-y")
        .arg("podman")
        .status()
        .context("Failed to run dnf install podman")?;

    if !status.success() {
        anyhow::bail!("dnf install podman failed");
    }

    Ok(())
}

fn install_with_yum() -> Result<()> {
    let status = StdCommand::new("sudo")
        .arg("yum")
        .arg("install")
        .arg("-y")
        .arg("podman")
        .status()
        .context("Failed to run yum install podman")?;

    if !status.success() {
        anyhow::bail!("yum install podman failed");
    }

    Ok(())
}

fn install_with_pacman() -> Result<()> {
    let status = StdCommand::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg("podman")
        .status()
        .context("Failed to run pacman -S podman")?;

    if !status.success() {
        anyhow::bail!("pacman -S podman failed");
    }

    Ok(())
}

fn install_with_zypper() -> Result<()> {
    let status = StdCommand::new("sudo")
        .arg("zypper")
        .arg("install")
        .arg("-y")
        .arg("podman")
        .status()
        .context("Failed to run zypper install podman")?;

    if !status.success() {
        anyhow::bail!("zypper install podman failed");
    }

    Ok(())
}

/// Podmanの初期化処理（インストール確認と必要に応じてインストール）
pub fn init_podman() -> Result<()> {
    if is_podman_installed() {
        let version = get_podman_version()?;
        println!("Podmanは既にインストールされています: {}", version);
        Ok(())
    } else {
        install_podman()
    }
}

