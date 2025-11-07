mod current_dir;
mod file_hash_index;
mod hash;
mod index_manager;
mod processor;
mod rust_parser;
mod scanner;
mod storage;

use index_manager::process_index;
use storage::Storage;

fn main() -> anyhow::Result<()> {
    let root_dir = current_dir::get_root_dir()?;

    println!("Scanning directory: {:?}", root_dir);

    // ディレクトリをスキャン
    let files = scanner::scan_directory(&root_dir)?;
    println!("Found {} files to process", files.len());

    // .overcodeディレクトリの準備
    let storage = Storage::new(&root_dir)?;

    // インデックスの処理を実行
    process_index(&storage, &files, &root_dir)?;

    println!("Processing complete!");
    Ok(())
}
