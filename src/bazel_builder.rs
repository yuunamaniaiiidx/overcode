use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use toml::Value;
use crate::storage::Storage;

/// BAZEL BUILDファイルとWORKSPACEファイルを生成する
pub fn generate_build_files(
    root_dir: &Path,
    history_path: &Path,
) -> Result<(PathBuf, PathBuf)> {
    // historyファイルを読み込む
    let content = fs::read_to_string(history_path)?;
    let value: Value = toml::from_str(&content)?;

    // buildsディレクトリを作成
    let overcode_dir = root_dir.join(".overcode");
    let builds_dir = overcode_dir.join("builds");
    
    // 既存のbuildsディレクトリをクリーンアップ（シンボリックリンクを削除）
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
        // builds/src/main.rs から ../{hash} へのリンク
        // builds/Cargo.lock から ../{hash} へのリンク
        let link_depth = path.matches('/').count();
        let relative_hash_path = if link_depth == 0 {
            // ルートファイル（例：Cargo.lock）の場合、../{hash}
            format!("../{}", hash)
        } else {
            // サブディレクトリのファイル（例：src/main.rs）の場合、../../{hash}
            format!("{}{}", "../".repeat(link_depth + 1), hash)
        };
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
            // buildsディレクトリがワークスペースルートなので、相対パスで参照
            build_content.push_str(&format!("        \"{}\",\n", normalized_path));
        }
        build_content.push_str("    ],\n");
        build_content.push_str("    visibility = [\"//visibility:public\"],\n");
        build_content.push_str(")\n\n");
    }

    // BUILDファイルを書き込む
    fs::write(&build_file_path, build_content)?;

    // WORKSPACEファイルを生成（builds/WORKSPACEに）
    let workspace_path = builds_dir.join("WORKSPACE");
    let workspace_content = "# Generated WORKSPACE file\n# DO NOT EDIT MANUALLY\n\nworkspace(name = \"overcode\")\n";
    fs::write(&workspace_path, workspace_content)?;

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

/// Buildコマンドの処理を実行する
/// 最新のhistoryファイルを取得し、BUILDファイルとWORKSPACEファイルを生成してBAZELビルドを実行する
pub fn process_build(root_dir: &Path) -> Result<()> {
    // 最新のhistoryファイルを取得
    let storage = Storage::new(root_dir)?;
    let latest_history = storage.get_latest_history_path()?;
    
    match latest_history {
        Some((_timestamp, history_path)) => {
            // BUILDファイルとWORKSPACEファイルを生成
            let (build_file_path, workspace_path) = generate_build_files(
                root_dir,
                &history_path,
            )?;
            
            println!("Generated BUILD file at: {:?}", build_file_path);
            println!("Generated WORKSPACE file at: {:?}", workspace_path);
            
            // BAZELコマンドを実行（.overcode/buildsディレクトリから）
            let builds_dir = root_dir.join(".overcode").join("builds");
            
            let output = ProcessCommand::new("bazel")
                .arg("build")
                .arg("//:sources")
                .current_dir(&builds_dir)
                .output()
                .context("Failed to execute bazel command")?;
            
            // 標準出力と標準エラー出力を表示
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
            
            if !output.status.success() {
                anyhow::bail!("BAZEL build failed with exit code: {:?}", output.status.code());
            }
            
            println!("BAZEL build completed successfully");
            Ok(())
        }
        None => {
            anyhow::bail!("No history file found. Please run 'index' command first.");
        }
    }
}

