use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use toml::Value;
use crate::file_index::FileIndex;

#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub paths: Vec<String>,
    pub hash: String,
    pub deps: Vec<(String, String)>, // (パス, sha256ハッシュ)のペア
}

pub struct Storage {
    overcode_dir: PathBuf,
}

impl Storage {
    pub fn new(root: &Path) -> anyhow::Result<Self> {
        let overcode_dir = root.join(".overcode");
        if !overcode_dir.exists() {
            fs::create_dir_all(&overcode_dir)?;
        }
        
        // historyディレクトリを作成
        let history_dir = overcode_dir.join("history");
        if !history_dir.exists() {
            fs::create_dir_all(&history_dir)?;
        }
        
        // blobsディレクトリを作成
        let blobs_dir = overcode_dir.join("blobs");
        if !blobs_dir.exists() {
            fs::create_dir_all(&blobs_dir)?;
        }
        
        Ok(Self { overcode_dir })
    }

    pub fn load_meta(&self, hash: &str) -> anyhow::Result<Vec<SourceEntry>> {
        let history_dir = self.overcode_dir.join("history");
        if !history_dir.exists() {
            return Ok(Vec::new());
        }

        // 最新のhistoryファイルを取得
        let latest_history = self.get_latest_history_path()?;
        let history_path = match latest_history {
            Some((_, path)) => path,
            None => return Ok(Vec::new()),
        };

        let content = fs::read_to_string(&history_path)?;
        let value: Value = toml::from_str(&content)?;

        let mut paths = Vec::new();
        let mut deps = Vec::new();
        
        if let Some(files) = value.get("files").and_then(|v| v.as_table()) {
            for (path, file_data) in files {
                if let Some(table) = file_data.as_table() {
                    let file_hash = table
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    
                    // 指定されたハッシュと一致するファイルのみを処理
                    if file_hash == hash {
                        paths.push(path.clone());
                        
                        // 最初に見つかったdepsを使用（同じハッシュのファイルは同じdepsを持つ）
                        if deps.is_empty() {
                            deps = table
                                .get("deps")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| {
                                            if let Some(table) = v.as_table() {
                                                let dep_path = table
                                                    .get("path")
                                                    .and_then(|p| p.as_str())
                                                    .map(|s| s.to_string())?;
                                                let dep_hash = table
                                                    .get("hash")
                                                    .and_then(|h| h.as_str())
                                                    .map(|s| s.to_string())
                                                    .unwrap_or_default();
                                                Some((dep_path, dep_hash))
                                            } else {
                                                v.as_str().map(|s| (s.to_string(), String::new()))
                                            }
                                        })
                                        .collect()
                                })
                                .unwrap_or_default();
                        }
                    }
                }
            }
        }

        if paths.is_empty() {
            Ok(Vec::new())
        } else {
            Ok(vec![SourceEntry {
                paths,
                hash: hash.to_string(),
                deps,
            }])
        }
    }

    pub fn save_file(&self, hash: &str, content: &[u8]) -> anyhow::Result<()> {
        let file_path = self.overcode_dir.join("blobs").join(hash);
        if !file_path.exists() {
            let mut file = fs::File::create(&file_path)?;
            file.write_all(content)?;
        }
        Ok(())
    }

    /// 最新の履歴ファイルのパスとタイムスタンプを取得する
    pub fn get_latest_history_path(&self) -> anyhow::Result<Option<(u64, PathBuf)>> {
        let history_dir = self.overcode_dir.join("history");
        if !history_dir.exists() {
            return Ok(None);
        }

        // historyディレクトリ内の.tomlファイルを列挙し、タイムスタンプが最大のものを探す
        let mut latest_file: Option<(u64, PathBuf)> = None;
        
        if let Ok(entries) = fs::read_dir(&history_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(timestamp) = file_stem.parse::<u64>() {
                                match latest_file {
                                    None => latest_file = Some((timestamp, path)),
                                    Some((latest_ts, _)) if timestamp > latest_ts => {
                                        latest_file = Some((timestamp, path))
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(latest_file)
    }

    /// 最新の履歴ファイルを読み込む
    /// パス→(mtime, size, hash)のマッピングを返す
    pub fn load_index(&self) -> anyhow::Result<FileIndex> {
        let history_dir = self.overcode_dir.join("history");
        if !history_dir.exists() {
            return Ok(FileIndex::new());
        }

        // historyディレクトリ内の.tomlファイルを列挙し、タイムスタンプが最大のものを探す
        let mut latest_file: Option<(u64, PathBuf)> = None;
        
        if let Ok(entries) = fs::read_dir(&history_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "toml" {
                        if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if let Ok(timestamp) = file_stem.parse::<u64>() {
                                match latest_file {
                                    None => latest_file = Some((timestamp, path)),
                                    Some((latest_ts, _)) if timestamp > latest_ts => {
                                        latest_file = Some((timestamp, path))
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        let index_path = match latest_file {
            Some((_, path)) => path,
            None => return Ok(FileIndex::new()),
        };

        let content = fs::read_to_string(&index_path)?;
        let value: Value = toml::from_str(&content)?;

        let mut index = HashMap::new();
        if let Some(files) = value.get("files").and_then(|v| v.as_table()) {
            for (path, file_data) in files {
                if let Some(table) = file_data.as_table() {
                    let mtime = table
                        .get("mtime")
                        .and_then(|v| v.as_integer())
                        .map(|i| i as u64)
                        .unwrap_or(0);
                    let size = table
                        .get("size")
                        .and_then(|v| v.as_integer())
                        .map(|i| i as u64)
                        .unwrap_or(0);
                    let hash = table
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    
                    // depsを読み込む（後方互換性のため、存在しない場合は空のベクター）
                    let deps = table
                        .get("deps")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| {
                                    if let Some(table) = v.as_table() {
                                        let dep_path = table
                                            .get("path")
                                            .and_then(|p| p.as_str())
                                            .map(|s| s.to_string())?;
                                        let dep_hash = table
                                            .get("hash")
                                            .and_then(|h| h.as_str())
                                            .map(|s| s.to_string())
                                            .unwrap_or_default();
                                        Some((dep_path, dep_hash))
                                    } else {
                                        // 後方互換性: 文字列のみの場合はパスのみとして扱う
                                        v.as_str().map(|s| (s.to_string(), String::new()))
                                    }
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    if !hash.is_empty() {
                        index.insert(path.clone(), (mtime, size, hash, deps));
                    }
                }
            }
        }

        Ok(FileIndex::from_hashmap(index))
    }

    /// インデックスファイルをhistoryディレクトリに保存する
    /// パス→(mtime, size, hash)のマッピングをhistory/{timestamp}.tomlとして保存
    pub fn save_index(&self, index: &FileIndex) -> anyhow::Result<()> {
        let history_dir = self.overcode_dir.join("history");
        
        // タイムスタンプを取得
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Failed to get timestamp: {}", e))?
            .as_secs();
        let history_path = history_dir.join(format!("{}.toml", timestamp));
        
        let mut toml_value = toml::map::Map::new();
        let mut files_table = toml::map::Map::new();

        for (path, (mtime, size, hash, deps)) in index.iter() {
            let mut file_table = toml::map::Map::new();
            file_table.insert("mtime".to_string(), Value::Integer(*mtime as i64));
            file_table.insert("size".to_string(), Value::Integer(*size as i64));
            file_table.insert("hash".to_string(), Value::String(hash.clone()));
            
            // depsを保存
            if !deps.is_empty() {
                file_table.insert("deps".to_string(), Value::Array(
                    deps.iter().map(|(dep_path, dep_hash)| {
                        let mut dep_table = toml::map::Map::new();
                        dep_table.insert("path".to_string(), Value::String(dep_path.clone()));
                        dep_table.insert("hash".to_string(), Value::String(dep_hash.clone()));
                        Value::Table(dep_table)
                    }).collect()
                ));
            }
            
            files_table.insert(path.clone(), Value::Table(file_table));
        }

        toml_value.insert("files".to_string(), Value::Table(files_table));
        let toml_string = toml::to_string_pretty(&Value::Table(toml_value))?;
        fs::write(&history_path, toml_string)?;
        Ok(())
    }
}

