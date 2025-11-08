use anyhow::Context;
use std::path::Path;
use crate::config;
use crate::file_hash_index::FileHashIndex;
use crate::file_index::FileIndex;
use crate::processor;
use crate::scanner;
use crate::storage::Storage;
use log::info;

/// インデックスの処理を実行する
/// 設定ファイルの読み込み、ディレクトリのスキャン、前回実行情報の読み込み、
/// ファイル処理、メタデータ更新、保存を行う
/// 戻り値: file_index
pub fn process_index(root_dir: &Path) -> anyhow::Result<FileIndex> {
    info!("Root directory: {:?}", root_dir);

    // 設定ファイルを読み込む
    let config = config::Config::load_from_root(root_dir)?;
    let ignore_patterns = config.get_ignore_patterns();
    if !ignore_patterns.is_empty() {
        info!("Loaded {} ignore pattern(s)", ignore_patterns.len());
    }

    info!("Scanning directory: {:?}", root_dir);

    // ディレクトリをスキャン
    let files = scanner::scan_directory(root_dir, &ignore_patterns)?;
    info!("Found {} files to process", files.len());

    // .overcodeディレクトリの準備
    let storage = Storage::new(root_dir)?;

    // 前回実行情報を取得（最新のindex_historyファイルから読み込む）
    let mut file_index = storage.load_index()
        .context("Failed to load latest index history file")?;
    info!("Loaded {} entries from latest index history file", file_index.len());

    // ファイル処理とハッシュ計算
    let file_hash_index = FileHashIndex::from_files(&files, &file_index)?;
    let (hash_to_info, path_to_hash, path_to_new_metadata) = file_hash_index.into_parts();

    // 全てのハッシュグループを処理
    file_index = processor::process_all_hash_groups(
        hash_to_info,
        &storage,
        root_dir,
        file_index,
        &path_to_new_metadata,
    )?;

    // 全てのファイルのメタデータを更新
    file_index = processor::update_all_file_metadata(
        file_index,
        &files,
        &path_to_hash,
    );

    // 現在のファイルリストに存在しないパスを削除
    file_index = processor::remove_obsolete_paths(&file_index, &files);

    // index_historyファイルとして保存
    storage.save_index(&file_index)
        .context("Failed to save index history file")?;

    Ok(file_index)
}

