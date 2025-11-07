use anyhow::{Context, Result};
use std::path::PathBuf;

/// コマンドライン引数からルートディレクトリを取得し、正規化する。
/// 引数が指定されていない場合は、現在の作業ディレクトリを返す。
pub fn get_root_dir() -> Result<PathBuf> {
    let args: Vec<String> = std::env::args().collect();
    let root_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        std::env::current_dir()?
    };
    
    root_dir.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", root_dir))
}

