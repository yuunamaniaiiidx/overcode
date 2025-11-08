use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

/// BAZEL BUILDファイルとWORKSPACEファイルを生成する
pub fn generate_build_files(
    root_dir: &Path,
    history_path: &Path,
) -> Result<(PathBuf, PathBuf)> {
    // historyファイルを読み込む
    let content = fs::read_to_string(history_path)?;
    let value: Value = toml::from_str(&content)?;

    // builds/v1ディレクトリを作成
    let overcode_dir = root_dir.join(".overcode");
    let builds_dir = overcode_dir.join("builds").join("v1");
    fs::create_dir_all(&builds_dir)?;

    // ファイルリストを収集（存在するファイルのみ）
    let mut source_files = Vec::new();
    if let Some(files) = value.get("files").and_then(|v| v.as_table()) {
        for (path, _) in files {
            // ファイルが実際に存在するか確認
            let file_path = root_dir.join(path);
            if file_path.exists() && file_path.is_file() {
                // パスをBAZEL形式に変換（相対パスとして扱う）
                // root_dirからの相対パスとして扱う
                source_files.push(path.clone());
            }
        }
    }

    // ルートパッケージにBUILDファイルを作成（または更新）
    let root_build_path = root_dir.join("BUILD");
    let mut root_build_content = String::new();
    root_build_content.push_str("# Generated BUILD file for root package\n");
    root_build_content.push_str("# DO NOT EDIT MANUALLY\n\n");
    
    // すべてのファイルをexports_filesでエクスポート
    if !source_files.is_empty() {
        root_build_content.push_str("exports_files([\n");
        for file in &source_files {
            let normalized_path = file.replace('\\', "/");
            root_build_content.push_str(&format!("    \"{}\",\n", normalized_path));
        }
        root_build_content.push_str("])\n");
    }
    
    fs::write(&root_build_path, root_build_content)?;

    // BUILDファイルのパス
    let build_file_path = builds_dir.join("BUILD");

    // BUILDファイルの内容を生成
    let mut build_content = String::new();
    build_content.push_str("# Generated BUILD file from history\n");
    build_content.push_str("# DO NOT EDIT MANUALLY\n\n");

    // filegroupルールを生成
    if !source_files.is_empty() {
        build_content.push_str("filegroup(\n");
        build_content.push_str("    name = \"sources\",\n");
        build_content.push_str("    srcs = [\n");
        for file in &source_files {
            // パスをBAZEL形式に変換（root_dirからの相対パス）
            // パス区切り文字を統一（バックスラッシュをスラッシュに）
            let normalized_path = file.replace('\\', "/");
            // BAZELのfilegroupでルートパッケージのファイルを参照する場合
            // ルートパッケージにBUILDファイルがあるため、`//:filename` の形式を使用
            build_content.push_str(&format!("        \"//:{}\",\n", normalized_path));
        }
        build_content.push_str("    ],\n");
        build_content.push_str("    visibility = [\"//visibility:public\"],\n");
        build_content.push_str(")\n\n");
    }

    // BUILDファイルを書き込む
    fs::write(&build_file_path, build_content)?;

    // WORKSPACEファイルを生成（root_dirに、既に存在する場合はスキップ）
    let workspace_path = root_dir.join("WORKSPACE");
    if !workspace_path.exists() {
        let workspace_content = "# Generated WORKSPACE file\n# DO NOT EDIT MANUALLY\n\nworkspace(name = \"overcode\")\n";
        fs::write(&workspace_path, workspace_content)?;
    }

    Ok((build_file_path, workspace_path))
}

