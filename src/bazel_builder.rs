use anyhow::Result;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use toml::Value;

/// BAZEL BUILDファイルとWORKSPACEファイルを生成する
pub fn generate_build_files(
    root_dir: &Path,
    history_path: &Path,
    timestamp: u64,
) -> Result<(PathBuf, PathBuf)> {
    // historyファイルを読み込む
    let content = fs::read_to_string(history_path)?;
    let value: Value = toml::from_str(&content)?;

    // builds/{timestamp}ディレクトリを作成
    let overcode_dir = root_dir.join(".overcode");
    let builds_dir = overcode_dir.join("builds").join(timestamp.to_string());
    
    // 既存のbuilds/{timestamp}ディレクトリをクリーンアップ（シンボリックリンクを削除）
    if builds_dir.exists() {
        remove_dir_contents(&builds_dir)?;
    }
    fs::create_dir_all(&builds_dir)?;

    // ファイルリストを収集（パスとハッシュのペア）
    let mut file_entries: Vec<(String, String)> = Vec::new();
    if let Some(files) = value.get("files").and_then(|v| v.as_table()) {
        for (path, file_data) in files {
            if let Some(table) = file_data.as_table() {
                if let Some(hash) = table.get("hash").and_then(|v| v.as_str()) {
                    if !hash.is_empty() {
                        // ハッシュファイルが存在するか確認
                        let hash_file_path = overcode_dir.join(hash);
                        if hash_file_path.exists() {
                            file_entries.push((path.clone(), hash.to_string()));
                        }
                    }
                }
            }
        }
    }

    // シンボリックリンクを作成（パス構造を復元）
    for (path, hash) in &file_entries {
        let link_path = builds_dir.join(path);
        
        // ディレクトリを作成
        if let Some(parent) = link_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 既存のリンクやファイルを削除
        if link_path.exists() || link_path.is_symlink() {
            fs::remove_file(&link_path).ok();
        }
        
        // 相対パスでシンボリックリンクを作成
        // builds/{timestamp}/src/main.rs から ../../{hash} へのリンク
        let link_depth = path.matches('/').count() + 1;
        let relative_hash_path = format!("{}../{}", "../".repeat(link_depth), hash);
        symlink(&relative_hash_path, &link_path)?;
    }

    // BUILDファイルのパス
    let build_file_path = builds_dir.join("BUILD");

    // BUILDファイルの内容を生成
    let mut build_content = String::new();
    build_content.push_str("# Generated BUILD file from history\n");
    build_content.push_str("# DO NOT EDIT MANUALLY\n\n");

    // filegroupルールを生成（シンボリックリンクされたファイルを参照）
    if !file_entries.is_empty() {
        build_content.push_str("filegroup(\n");
        build_content.push_str("    name = \"sources\",\n");
        build_content.push_str("    srcs = [\n");
        for (path, _) in &file_entries {
            // パスをBAZEL形式に変換（パス区切り文字を統一）
            let normalized_path = path.replace('\\', "/");
            // {timestamp}ディレクトリがワークスペースルートなので、相対パスで参照
            build_content.push_str(&format!("        \"{}\",\n", normalized_path));
        }
        build_content.push_str("    ],\n");
        build_content.push_str("    visibility = [\"//visibility:public\"],\n");
        build_content.push_str(")\n\n");
    }

    // BUILDファイルを書き込む
    fs::write(&build_file_path, build_content)?;

    // WORKSPACEファイルを生成（.overcode/WORKSPACEに）
    let workspace_path = overcode_dir.join("WORKSPACE");
    let workspace_content = "# Generated WORKSPACE file\n# DO NOT EDIT MANUALLY\n\nworkspace(name = \"overcode\")\n";
    fs::write(&workspace_path, workspace_content)?;
    
    // builds/{timestamp}ディレクトリにもWORKSPACEファイルを配置（BAZELがbuilds/{timestamp}から実行されるため）
    let workspace_timestamp_path = builds_dir.join("WORKSPACE");
    fs::write(&workspace_timestamp_path, workspace_content)?;

    Ok((build_file_path, workspace_path))
}

/// ディレクトリの内容を再帰的に削除（シンボリックリンクも含む）
fn remove_dir_contents(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_symlink() || path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(&path)?;
        }
    }
    
    Ok(())
}

