mod current_dir;
mod file_hash_index;
mod hash;
mod rust_parser;
mod scanner;
mod storage;

use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::time::UNIX_EPOCH;
use file_hash_index::FileHashIndex;
use storage::{SourceEntry, Storage};

fn main() -> anyhow::Result<()> {
    let root_dir = current_dir::get_root_dir()?;

    println!("Scanning directory: {:?}", root_dir);

    // ディレクトリをスキャン
    let files = scanner::scan_directory(&root_dir)?;
    println!("Found {} files to process", files.len());

    // .overcodeディレクトリの準備
    let storage = Storage::new(&root_dir)?;

    // 前回実行情報を取得（index.tomlから読み込む）
    let mut path_to_metadata = storage.load_index()
        .context("Failed to load index.toml")?;
    println!("Loaded {} entries from index.toml", path_to_metadata.len());

    // ファイル処理とハッシュ計算
    let file_hash_index = FileHashIndex::from_files(&files, &path_to_metadata)?;
    let (hash_to_info, path_to_hash, path_to_new_metadata) = file_hash_index.into_parts();

    // 各ハッシュについて処理
    for (hash, (paths, file_path)) in hash_to_info {
        // .overcode内にメタファイルが存在するか確認（ファイルシステムで直接確認）
        let meta_exists = storage.meta_exists(&hash);
        
        // 既存のメタ情報を読み込む（存在する場合のみ）
        let existing_entries = if meta_exists {
            storage.load_meta(&hash).unwrap_or_default()
        } else {
            Vec::new()
        };
        
        // 既存のエントリから全ての既知のパスを収集
        let known_paths: std::collections::HashSet<String> = existing_entries
            .iter()
            .flat_map(|e| e.paths.iter().cloned())
            .collect();

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

            // 依存関係を解析（Rustファイルの場合、最初のパスを使用）
            let mut deps = Vec::new();
            if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content_str = String::from_utf8_lossy(&content);
                let dep_paths = rust_parser::extract_dependencies(
                    &file_path,
                    &content_str,
                    &root_dir,
                )
                .unwrap_or_default();
                
                // 各依存先のハッシュを計算
                for dep_path in dep_paths {
                    let dep_full_path = root_dir.join(&dep_path);
                    let dep_hash = if let Some((_, _, cached_hash)) = path_to_metadata.get(&dep_path) {
                        // index.tomlから既存のハッシュを取得
                        cached_hash.clone()
                    } else if dep_full_path.exists() && dep_full_path.is_file() {
                        // ファイルの場合、ハッシュを計算
                        hash::calculate_file_hash(&dep_full_path)
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };
                    deps.push((dep_path, dep_hash));
                }
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
                    paths: paths.clone(),
                    hash: hash.clone(),
                    deps,
                });
            }

            // index.toml用のマッピングを更新
            for path in &paths {
                if let Some((mtime, size)) = path_to_new_metadata.get(path) {
                    path_to_metadata.insert(path.clone(), (*mtime, *size, hash.clone()));
                }
            }

            // メタ情報を保存
            storage.save_meta(&hash, &updated_entries)
                .with_context(|| format!("Failed to save meta for hash: {}", hash))?;
        }
    }

    // index.tomlを更新（処理完了後）
    // 全てのファイルについて、現在のmtime/sizeでpath_to_metadataを更新
    // ハッシュ計算をスキップしたパスも含めて、全てのパスを更新
    for file_entry in &files {
        let relative_path_str = file_entry.relative_path.to_string_lossy().to_string();
        
        // 現在のメタデータを取得
        if let Ok(metadata) = fs::metadata(&file_entry.path) {
            let mtime = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let size = metadata.len();
            
            // path_to_metadataからハッシュを取得
            if let Some((_, _, hash)) = path_to_metadata.get(&relative_path_str) {
                // 既に存在する場合は、mtime/sizeを更新
                path_to_metadata.insert(relative_path_str, (mtime, size, hash.clone()));
            } else if let Some(hash) = path_to_hash.get(&relative_path_str) {
                // 新規に計算したハッシュのパスの場合
                path_to_metadata.insert(relative_path_str, (mtime, size, hash.clone()));
            }
        }
    }

    // 現在のファイルリストに存在しないパスをindex.tomlから削除
    let current_paths: std::collections::HashSet<String> = files
        .iter()
        .map(|f| f.relative_path.to_string_lossy().to_string())
        .collect();
    
    path_to_metadata.retain(|path, _| current_paths.contains(path));

    // index.tomlを保存
    storage.save_index(&path_to_metadata)
        .context("Failed to save index.toml")?;

    println!("Processing complete!");
    Ok(())
}
