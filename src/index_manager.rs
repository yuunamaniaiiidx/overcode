use anyhow::Context;
use std::collections::HashMap;
use std::path::Path;
use crate::file_hash_index::FileHashIndex;
use crate::processor;
use crate::scanner::FileEntry;
use crate::storage::Storage;

/// インデックスの処理を実行する
/// 前回実行情報の読み込み、ファイル処理、メタデータ更新、保存を行う
pub fn process_index(
    storage: &Storage,
    files: &[FileEntry],
    root_dir: &Path,
) -> anyhow::Result<()> {
    // 前回実行情報を取得（index.tomlから読み込む）
    let mut path_to_metadata = storage.load_index()
        .context("Failed to load index.toml")?;
    println!("Loaded {} entries from index.toml", path_to_metadata.len());

    // ファイル処理とハッシュ計算
    let file_hash_index = FileHashIndex::from_files(&files, &path_to_metadata)?;
    let (hash_to_info, path_to_hash, path_to_new_metadata) = file_hash_index.into_parts();

    // 全てのハッシュグループを処理
    path_to_metadata = processor::process_all_hash_groups(
        hash_to_info,
        &storage,
        &root_dir,
        path_to_metadata,
        &path_to_new_metadata,
    )?;

    // 全てのファイルのメタデータを更新
    path_to_metadata = processor::update_all_file_metadata(
        path_to_metadata,
        &files,
        &path_to_hash,
    );

    // 現在のファイルリストに存在しないパスをindex.tomlから削除
    path_to_metadata = processor::remove_obsolete_paths(&path_to_metadata, &files);

    // index.tomlを保存
    storage.save_index(&path_to_metadata)
        .context("Failed to save index.toml")?;

    Ok(())
}

