use anyhow::Context;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::storage::Storage;
use log::{info, warn};

/// build処理を実行する
/// 最新のindex_historyデータをもとに、.overcode/builds配下に元のファイル構造を再現する
pub fn process_build(root_dir: &Path) -> anyhow::Result<()> {
    let storage = Storage::new(root_dir)?;

    // 最新のindex_historyファイルを取得
    let index = storage.load_index()
        .context("Failed to load latest index history file")?;
    
    if index.len() == 0 {
        anyhow::bail!("No index history found. Please run 'index' command first.");
    }

    // buildsディレクトリを作成
    let builds_dir = root_dir.join(".overcode").join("builds");
    if !builds_dir.exists() {
        fs::create_dir_all(&builds_dir)?;
    }

    // build_historyディレクトリを作成
    let build_history_dir = root_dir.join(".overcode").join("build_history");
    if !build_history_dir.exists() {
        fs::create_dir_all(&build_history_dir)?;
    }

    // 最新のbuild_historyを読み込む（スキップ判定用）
    let build_history = storage.load_build_history()
        .context("Failed to load build history")?;

    let blobs_dir = root_dir.join(".overcode").join("blobs");
    let mut copied = 0;
    let mut skipped = 0;
    let mut build_files = HashMap::new();

    // 各ファイルについて処理
    for (path, (_mtime, size, hash, _deps)) in index.iter() {
        let blob_path = blobs_dir.join(hash);
        if !blob_path.exists() {
            warn!("blob not found for hash {} (path: {})", hash, path);
            continue;
        }

        // builds配下のファイルパスを作成
        let build_file_path = builds_dir.join(path);
        
        // 親ディレクトリを作成
        if let Some(parent) = build_file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 既存ファイルがある場合、build_historyの最新データとmtime/sizeを比較
        let should_skip = if let Some((history_mtime, history_size, history_hash)) = build_history.get(path) {
            // build_historyに記録があり、hash/sizeが一致し、ファイルが存在する場合はmtime/sizeをチェック
            if history_hash == hash && *history_size == *size && build_file_path.exists() {
                if let Ok(metadata) = fs::metadata(&build_file_path) {
                    let existing_size = metadata.len();
                    let existing_mtime = metadata
                        .modified()?
                        .duration_since(UNIX_EPOCH)
                        .map_err(|e| anyhow::anyhow!("Failed to get mtime: {}", e))?
                        .as_secs();
                    
                    existing_size == *history_size && existing_mtime == *history_mtime
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if should_skip {
            skipped += 1;
            // スキップしたファイルもbuild_historyに記録するため、既存のmtimeを取得
            if let Ok(metadata) = fs::metadata(&build_file_path) {
                let existing_mtime = metadata
                    .modified()?
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| anyhow::anyhow!("Failed to get mtime: {}", e))?
                    .as_secs();
                build_files.insert(path.clone(), (existing_mtime, *size, hash.clone()));
            }
            continue;
        }

        // ファイルをコピー（mtimeは自動的に現在時刻になる）
        fs::copy(&blob_path, &build_file_path)?;
        
        // コピー後のmtimeを取得
        let metadata = fs::metadata(&build_file_path)?;
        let copied_mtime = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get mtime: {}", e))?
            .as_secs();
        
        build_files.insert(path.clone(), (copied_mtime, *size, hash.clone()));
        copied += 1;
    }

    // build_historyに保存
    storage.save_build_history(&build_files)
        .context("Failed to save build history")?;

    info!("Build completed: {} files copied, {} files skipped", copied, skipped);
    Ok(())
}

