mod current_dir;
mod file_hash_index;
mod hash;
mod processor;
mod rust_parser;
mod scanner;
mod storage;

use anyhow::Context;
use file_hash_index::FileHashIndex;
use storage::Storage;

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

    println!("Processing complete!");
    Ok(())
}
