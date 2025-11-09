use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use crate::config::{Config, IgnorePattern};

pub struct FileEntry {
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

pub fn scan_directory(root: &Path, ignore_patterns: &[IgnorePattern], config: &Config) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    
    // WalkBuilderを構築
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(false)  // デフォルトの.gitignoreは無効化
        .git_exclude(true);
    
    // 設定から読み込んだignoreファイルを追加
    let ignore_files = config.get_ignore_files();
    for ignore_file in ignore_files {
        let ignore_path = root.join(&ignore_file);
        if ignore_path.exists() {
            // WalkBuilderにignoreファイルを追加
            if let Some(err) = builder.add_ignore(&ignore_path) {
                return Err(anyhow::anyhow!("Failed to add ignore file {:?}: {}", ignore_path, err));
            }
        }
    }
    
    let walker = builder.build();
    
    for result in walker {
        let entry = result?;
        let path = entry.path();
        
        // .overcodeディレクトリを除外
        if path.components().any(|c| c.as_os_str() == ".overcode") {
            continue;
        }
        
        // ignoreパターンで除外
        let should_ignore = ignore_patterns.iter().any(|pattern| pattern.matches(path, root));
        if should_ignore {
            continue;
        }
        
        if path.is_file() {
            let relative_path = path.strip_prefix(root)?
                .to_path_buf();
            entries.push(FileEntry {
                path: path.to_path_buf(),
                relative_path,
            });
        }
    }
    
    Ok(entries)
}

