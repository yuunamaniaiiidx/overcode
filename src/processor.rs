use anyhow::Context;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use crate::file_index::FileIndex;
use crate::hash;
use crate::rust_parser;
use crate::scanner::FileEntry;
use crate::storage::Storage;
use log::info;

/// パスリストをソートして重複を除去する
pub fn normalize_paths(mut paths: Vec<String>) -> Vec<String> {
    paths.sort();
    paths.dedup();
    paths
}

/// 既存のメタ情報を読み込み、既知のパスを収集する
pub fn load_existing_paths(
    storage: &Storage,
    hash: &str,
) -> anyhow::Result<HashSet<String>> {
    let entries = storage.load_meta(hash).unwrap_or_default();
    let known_paths: HashSet<String> = entries
        .iter()
        .flat_map(|e| e.paths.iter().cloned())
        .collect();
    Ok(known_paths)
}

/// 新しいパスがあるかチェックする
pub fn has_new_paths(paths: &[String], known_paths: &HashSet<String>) -> bool {
    paths.iter().any(|p| !known_paths.contains(p))
}

/// 依存関係を解析し、各依存先のハッシュを計算する
pub fn extract_dependencies_with_hashes(
    file_path: &PathBuf,
    content: &[u8],
    root_dir: &Path,
    file_index: &FileIndex,
) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    
    if file_path.extension().and_then(|s| s.to_str()) == Some("rs") {
        let content_str = String::from_utf8_lossy(content);
        let dep_paths = rust_parser::extract_dependencies(
            file_path,
            &content_str,
            root_dir,
        )
        .unwrap_or_default();
        
        // 各依存先のハッシュを計算
        for dep_path in dep_paths {
            let dep_full_path = root_dir.join(&dep_path);
            let dep_hash = if let Some((_, _, cached_hash, _)) = file_index.get(&dep_path) {
                // 既存のハッシュを取得
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
    
    deps
}

/// マッピングを更新する（新しいFileIndexを返す）
pub fn update_path_to_metadata(
    file_index: &FileIndex,
    paths: &[String],
    path_to_new_metadata: &HashMap<String, (u64, u64)>,
    hash: &str,
    deps: Vec<(String, String)>,
) -> FileIndex {
    let mut result = file_index.clone();
    for path in paths {
        if let Some((mtime, size)) = path_to_new_metadata.get(path) {
            result.insert(path.clone(), (*mtime, *size, hash.to_string(), deps.clone()));
        }
    }
    result
}

/// ファイルのメタデータ（mtime, size）を取得する
pub fn get_file_metadata(path: &Path) -> Option<(u64, u64)> {
    fs::metadata(path).ok().and_then(|metadata| {
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let size = metadata.len();
        Some((mtime, size))
    })
}

/// file_indexを更新する（新しいFileIndexを返す）
pub fn update_path_metadata(
    file_index: &FileIndex,
    path: String,
    mtime: u64,
    size: u64,
    path_to_hash: &HashMap<String, String>,
) -> FileIndex {
    let mut result = file_index.clone();
    if let Some((_, _, hash, deps)) = result.get(&path) {
        // 既に存在する場合は、mtime/sizeを更新（depsは保持）
        result.insert(path.clone(), (mtime, size, hash.clone(), deps.clone()));
    } else if let Some(hash) = path_to_hash.get(&path) {
        // 新規に計算したハッシュのパスの場合（depsは空）
        result.insert(path, (mtime, size, hash.clone(), Vec::new()));
    }
    result
}

/// 現在のファイルリストに存在しないパスを削除する（新しいFileIndexを返す）
pub fn remove_obsolete_paths(
    file_index: &FileIndex,
    files: &[FileEntry],
) -> FileIndex {
    let current_paths: HashSet<String> = files
        .iter()
        .map(|f| f.relative_path.to_string_lossy().to_string())
        .collect();
    
    file_index
        .iter()
        .filter(|(path, _)| current_paths.contains(*path))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

/// ハッシュごとの処理を実行する（更新されたfile_indexを返す）
pub fn process_hash_group(
    hash: String,
    mut paths: Vec<String>,
    file_path: PathBuf,
    storage: &Storage,
    root_dir: &Path,
    file_index: &FileIndex,
    path_to_new_metadata: &HashMap<String, (u64, u64)>,
) -> anyhow::Result<FileIndex> {
    // パスをソートして重複を除去
    paths = normalize_paths(paths);

    // 既存のメタ情報を読み込み、既知のパスを収集
    let known_paths = load_existing_paths(storage, &hash)?;

    // 新しいパスがあるかチェック
    let has_new_paths = has_new_paths(&paths, &known_paths);

    // 新しいハッシュ、または新しいパスの場合のみ処理
    let updated_metadata = if known_paths.is_empty() || has_new_paths {
        info!("Processing hash: {} ({} paths)", &hash[..8], paths.len());

        // ファイル内容を読み込む（同じハッシュなら内容は同じなので1つだけ読み込む）
        let content = fs::read(&file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;

        // ファイルを保存
        storage.save_file(&hash, &content)
            .with_context(|| format!("Failed to save file for hash: {}", hash))?;

        // 依存関係を解析
        let deps = extract_dependencies_with_hashes(
            &file_path,
            &content,
            root_dir,
            file_index,
        );

        // マッピングを更新（新しいFileIndexを返す）
        let updated_metadata = update_path_to_metadata(
            file_index,
            &paths,
            path_to_new_metadata,
            &hash,
            deps,
        );

        updated_metadata
    } else {
        file_index.clone()
    };

    Ok(updated_metadata)
}

/// 全てのハッシュグループを処理する
pub fn process_all_hash_groups(
    hash_to_info: impl IntoIterator<Item = (String, (Vec<String>, PathBuf))>,
    storage: &Storage,
    root_dir: &Path,
    mut file_index: FileIndex,
    path_to_new_metadata: &HashMap<String, (u64, u64)>,
) -> anyhow::Result<FileIndex> {
    for (hash, (paths, file_path)) in hash_to_info {
        file_index = process_hash_group(
            hash,
            paths,
            file_path,
            storage,
            root_dir,
            &file_index,
            path_to_new_metadata,
        )?;
    }
    Ok(file_index)
}

/// 全てのファイルのメタデータを更新する
/// ハッシュ計算をスキップしたパスも含めて、全てのパスを更新
pub fn update_all_file_metadata(
    file_index: FileIndex,
    files: &[FileEntry],
    path_to_hash: &HashMap<String, String>,
) -> FileIndex {
    files
        .iter()
        .fold(file_index, |mut index, file_entry| {
            let relative_path_str = file_entry.relative_path.to_string_lossy().to_string();
            
            if let Some((mtime, size)) = get_file_metadata(&file_entry.path) {
                index = update_path_metadata(
                    &index,
                    relative_path_str,
                    mtime,
                    size,
                    path_to_hash,
                );
            }
            index
        })
}

