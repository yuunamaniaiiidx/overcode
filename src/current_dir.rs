use anyhow::{Context, Result};
use std::path::PathBuf;

/// ルートディレクトリを取得し、正規化する。
/// 引数が指定されている場合はそのパスを使用し、指定されていない場合は現在の作業ディレクトリを返す。
pub fn get_root_dir(dir: Option<&PathBuf>) -> Result<PathBuf> {
    let root_dir = if let Some(d) = dir {
        d.clone()
    } else {
        std::env::current_dir()?
    };
    
    root_dir.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", root_dir))
}

