use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub path: PathBuf,
    pub relative_path: PathBuf,
}

pub fn scan_directory(root: &Path) -> anyhow::Result<Vec<FileEntry>> {
    let mut entries = Vec::new();
    
    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .build();
    
    for result in walker {
        let entry = result?;
        let path = entry.path();
        
        // .overcodeディレクトリを除外
        if path.components().any(|c| c.as_os_str() == ".overcode") {
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

