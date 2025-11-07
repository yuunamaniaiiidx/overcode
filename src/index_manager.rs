use anyhow::Context;
use std::path::Path;
use crate::file_hash_index::FileHashIndex;
use crate::file_index::FileIndex;
use crate::processor;
use crate::scanner::FileEntry;
use crate::storage::Storage;

/// インデックスの処理を実行する
/// 前回実行情報の読み込み、ファイル処理、メタデータ更新、保存を行う
/// 戻り値: file_index
pub fn process_index(
    storage: &Storage,
    files: &[FileEntry],
    root_dir: &Path,
) -> anyhow::Result<FileIndex> {
    // 前回実行情報を取得（index.tomlから読み込む）
    let mut file_index = storage.load_index()
        .context("Failed to load index.toml")?;
    println!("Loaded {} entries from index.toml", file_index.len());

    // ファイル処理とハッシュ計算
    let file_hash_index = FileHashIndex::from_files(&files, &file_index)?;
    let (hash_to_info, path_to_hash, path_to_new_metadata) = file_hash_index.into_parts();

    // 全てのハッシュグループを処理
    file_index = processor::process_all_hash_groups(
        hash_to_info,
        &storage,
        &root_dir,
        file_index,
        &path_to_new_metadata,
    )?;

    // 全てのファイルのメタデータを更新
    file_index = processor::update_all_file_metadata(
        file_index,
        &files,
        &path_to_hash,
    );

    // 現在のファイルリストに存在しないパスをindex.tomlから削除
    file_index = processor::remove_obsolete_paths(&file_index, &files);

    // index.tomlを保存
    storage.save_index(&file_index)
        .context("Failed to save index.toml")?;

    Ok(file_index)
}

