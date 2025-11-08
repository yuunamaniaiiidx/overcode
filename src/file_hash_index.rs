use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;
use crate::file_index::FileIndex;
use crate::hash;
use crate::scanner::FileEntry;

pub struct FileHashIndex {
    hash_to_info: HashMap<String, (Vec<String>, PathBuf)>,
    path_to_new_metadata: HashMap<String, (u64, u64)>,
    path_to_hash: HashMap<String, String>,
}

impl FileHashIndex {
    pub fn new() -> Self {
        Self {
            hash_to_info: HashMap::new(),
            path_to_new_metadata: HashMap::new(),
            path_to_hash: HashMap::new(),
        }
    }

    /// ファイルリストからFileHashIndexを作成する。
    pub fn from_files(
        files: &[FileEntry],
        file_index: &FileIndex,
    ) -> Result<Self> {
        files.iter()
            .try_fold(Self::new(), |mut index, file_entry| {
                index.process_file(file_entry, file_index)?;
                Ok(index)
            })
    }

    /// ファイルを処理し、メタデータを取得してハッシュ計算の必要性を判断する。
    /// 必要に応じてハッシュを計算し、データ構造を更新する。
    pub fn process_file(
        &mut self,
        file_entry: &FileEntry,
        file_index: &FileIndex,
    ) -> Result<()> {
        let relative_path_str = file_entry.relative_path.to_string_lossy().to_string();

        // ファイルのメタデータを取得（ファイルを開かずに）
        let metadata = fs::metadata(&file_entry.path)
            .with_context(|| format!("Failed to get metadata for {:?}", file_entry.path))?;
        let mtime = metadata
            .modified()?
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let size = metadata.len();

        // 既存のメタデータと比較
        let should_calculate_hash = if let Some((cached_mtime, cached_size, cached_hash, _deps)) =
            file_index.get(&relative_path_str)
        {
            // mtimeとsizeが一致しない場合はハッシュを再計算
            if *cached_mtime != mtime || *cached_size != size {
                true
            } else {
                // 変更なし - キャッシュされたハッシュを使用
                let hash = cached_hash.clone();
                let entry = self
                    .hash_to_info
                    .entry(hash)
                    .or_insert_with(|| (Vec::new(), file_entry.path.clone()));
                entry.0.push(relative_path_str.clone());
                self.path_to_new_metadata
                    .insert(relative_path_str.clone(), (mtime, size));
                false
            }
        } else {
            // 新規ファイル - ハッシュを計算
            true
        };

        if should_calculate_hash {
            // ハッシュを計算
            let hash = hash::calculate_file_hash(&file_entry.path)
                .with_context(|| format!("Failed to calculate hash for {:?}", file_entry.path))?;

            // ハッシュごとにパスを集約
            let entry = self
                .hash_to_info
                .entry(hash.clone())
                .or_insert_with(|| (Vec::new(), file_entry.path.clone()));
            entry.0.push(relative_path_str.clone());
            self.path_to_new_metadata
                .insert(relative_path_str.clone(), (mtime, size));
            self.path_to_hash.insert(relative_path_str, hash);
        }

        Ok(())
    }

    pub fn into_parts(
        self,
    ) -> (
        HashMap<String, (Vec<String>, PathBuf)>,
        HashMap<String, String>,
        HashMap<String, (u64, u64)>,
    ) {
        // パスリストを正規化（ソートと重複除去）
        let normalized_hash_to_info = self.hash_to_info
            .into_iter()
            .map(|(hash, (mut paths, file_path))| {
                paths.sort();
                paths.dedup();
                (hash, (paths, file_path))
            })
            .collect();
        
        (normalized_hash_to_info, self.path_to_hash, self.path_to_new_metadata)
    }
}

