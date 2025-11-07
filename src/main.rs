mod hash;
mod python_parser;
mod scanner;
mod storage;

use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use storage::{SourceEntry, Storage};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let root_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };

    let root_dir = root_dir.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", root_dir))?;

    println!("Scanning directory: {:?}", root_dir);

    // .overcodeディレクトリの準備
    let storage = Storage::new(&root_dir)?;

    // 既存のメタ情報を読み込み
    let known_hashes = storage.get_all_known_hashes()?;
    println!("Found {} known file hashes", known_hashes.len());

    // ディレクトリをスキャン
    let files = scanner::scan_directory(&root_dir)?;
    println!("Found {} files to process", files.len());

    // ハッシュ→パスのマッピング（現在のスキャン結果）
    // ハッシュ→(パスリスト, ファイルパス)のマッピング
    let mut hash_to_info: HashMap<String, (Vec<String>, PathBuf)> = HashMap::new();

    // まず全てのファイルのハッシュを計算
    for file_entry in &files {
        let relative_path_str = file_entry.relative_path.to_string_lossy().to_string();
        
        // ハッシュを計算
        let hash = hash::calculate_file_hash(&file_entry.path)
            .with_context(|| format!("Failed to calculate hash for {:?}", file_entry.path))?;

        // ハッシュごとにパスを集約
        let entry = hash_to_info.entry(hash).or_insert_with(|| (Vec::new(), file_entry.path.clone()));
        entry.0.push(relative_path_str);
    }

    // 各ハッシュについて処理
    for (hash, (paths, file_path)) in hash_to_info {
        // パスをソートして重複を除去
        let mut paths = paths;
        paths.sort();
        paths.dedup();

        // 既存のメタ情報を確認
        let existing_entries = known_hashes.get(&hash).cloned().unwrap_or_default();
        
        // 既存のエントリから全ての既知のパスを収集
        let mut known_paths: Vec<String> = existing_entries
            .iter()
            .flat_map(|e| e.paths.iter().cloned())
            .collect();
        known_paths.sort();
        known_paths.dedup();

        // 新しいパスがあるかチェック
        let has_new_paths = paths.iter().any(|p| !known_paths.contains(p));

        // 新しいハッシュ、または新しいパスの場合のみ処理
        if existing_entries.is_empty() || has_new_paths {
            println!("Processing hash: {} ({} paths)", &hash[..8], paths.len());

            // ファイル内容を読み込む（同じハッシュなら内容は同じなので1つだけ読み込む）
            let content = fs::read(&file_path)
                .with_context(|| format!("Failed to read file: {:?}", file_path))?;

            // ファイルを保存
            storage.save_file(&hash, &content)
                .with_context(|| format!("Failed to save file for hash: {}", hash))?;

            // 依存関係を解析（Pythonファイルの場合、最初のパスを使用）
            let mut deps = Vec::new();
            if file_path.extension().and_then(|s| s.to_str()) == Some("py") {
                let content_str = String::from_utf8_lossy(&content);
                deps = python_parser::extract_dependencies(
                    &file_path,
                    &content_str,
                    &root_dir,
                )
                .unwrap_or_default();
            }

            // メタ情報を更新
            // 既存のエントリをマージするか、新しいエントリを作成
            let mut updated_entries = existing_entries;
            
            // 既存のエントリに同じハッシュのものがあるかチェック
            let mut found = false;
            for entry in &mut updated_entries {
                if entry.hash == hash {
                    // 既存のパスと新しいパスをマージ
                    let mut all_paths = entry.paths.clone();
                    all_paths.extend(paths.clone());
                    all_paths.sort();
                    all_paths.dedup();
                    entry.paths = all_paths;
                    // 依存関係を更新（最新のものを使用）
                    entry.deps = deps.clone();
                    found = true;
                    break;
                }
            }

            // 新しいエントリを追加
            if !found {
                updated_entries.push(SourceEntry {
                    paths,
                    hash: hash.clone(),
                    deps,
                });
            }

            // メタ情報を保存
            storage.save_meta(&hash, &updated_entries)
                .with_context(|| format!("Failed to save meta for hash: {}", hash))?;
        }
    }

    println!("Processing complete!");
    Ok(())
}
