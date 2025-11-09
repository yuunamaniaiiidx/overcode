use crate::storage::Storage;
use crate::file_index::FileIndex;
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

/// Storageがbuild.rsから期待される振る舞いをテストする
#[test]
fn test_storage_new_creates_directories() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    // Storage::new()がディレクトリを作成することを確認
    let _storage = Storage::new(root_dir).unwrap();
    
    // .overcodeディレクトリが作成されている
    let overcode_dir = root_dir.join(".overcode");
    assert!(overcode_dir.exists());
    
    // index_historyディレクトリが作成されている
    let history_dir = overcode_dir.join("index_history");
    assert!(history_dir.exists());
    
    // blobsディレクトリが作成されている
    let blobs_dir = overcode_dir.join("blobs");
    assert!(blobs_dir.exists());
}

#[test]
fn test_storage_load_index_returns_empty_when_no_history() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // 履歴がない場合、空のFileIndexが返される
    let index = storage.load_index().unwrap();
    assert_eq!(index.len(), 0);
}

#[test]
fn test_storage_load_index_loads_correct_data() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // テスト用のインデックスを作成して保存
    let mut test_index = FileIndex::new();
    test_index.insert(
        "test.rs".to_string(),
        (1000, 100, "abc123".to_string(), Vec::new())
    );
    storage.save_index(&test_index).unwrap();
    
    // インデックスを読み込む
    let loaded_index = storage.load_index().unwrap();
    assert_eq!(loaded_index.len(), 1);
    
    // データが正しく読み込まれている
    let metadata = loaded_index.get("test.rs").unwrap();
    assert_eq!(metadata.0, 1000); // mtime
    assert_eq!(metadata.1, 100);  // size
    assert_eq!(metadata.2, "abc123"); // hash
}

#[test]
fn test_storage_load_build_history_returns_empty_when_no_history() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // build_historyがない場合、空のHashMapが返される
    let build_history = storage.load_build_history().unwrap();
    assert_eq!(build_history.len(), 0);
}

#[test]
fn test_storage_save_and_load_build_history() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // テスト用のbuild_historyを作成
    let mut build_files = HashMap::new();
    build_files.insert("test.rs".to_string(), (2000, 200, "def456".to_string()));
    
    // build_historyを保存
    storage.save_build_history(&build_files).unwrap();
    
    // build_historyを読み込む
    let loaded_history = storage.load_build_history().unwrap();
    assert_eq!(loaded_history.len(), 1);
    
    // データが正しく読み込まれている
    let metadata = loaded_history.get("test.rs").unwrap();
    assert_eq!(metadata.0, 2000); // mtime
    assert_eq!(metadata.1, 200);  // size
    assert_eq!(metadata.2, "def456"); // hash
}

#[test]
fn test_storage_load_build_history_returns_latest() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // 最初のbuild_historyを保存
    let mut build_files1 = HashMap::new();
    build_files1.insert("old.rs".to_string(), (1000, 100, "old_hash".to_string()));
    storage.save_build_history(&build_files1).unwrap();
    
    // 少し待ってから（タイムスタンプが異なるように）
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // 2番目のbuild_historyを保存
    let mut build_files2 = HashMap::new();
    build_files2.insert("new.rs".to_string(), (2000, 200, "new_hash".to_string()));
    storage.save_build_history(&build_files2).unwrap();
    
    // 最新のbuild_historyが読み込まれることを確認
    let loaded_history = storage.load_build_history().unwrap();
    assert_eq!(loaded_history.len(), 1);
    assert!(loaded_history.contains_key("new.rs"));
    assert!(!loaded_history.contains_key("old.rs"));
}

#[test]
fn test_storage_save_file_creates_blob() {
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path();
    
    let storage = Storage::new(root_dir).unwrap();
    
    // ファイルを保存
    let content = b"test content";
    let hash = "test_hash";
    storage.save_file(hash, content).unwrap();
    
    // blobファイルが作成されている
    let blob_path = root_dir.join(".overcode").join("blobs").join(hash);
    assert!(blob_path.exists());
    
    // 内容が正しい
    let saved_content = fs::read(&blob_path).unwrap();
    assert_eq!(saved_content, content);
}

