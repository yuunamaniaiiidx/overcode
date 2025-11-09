#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::run::process_run;

    #[test]
    fn test_process_run_without_config() {
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルが存在しない場合
        let result = process_run(temp_dir.path(), &[]);
        
        // command.runセクションが見つからないため、エラーが返されるはず
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("command.run") || error_msg.contains("not found"));
    }

    #[test]
    fn test_process_run_with_incomplete_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.runセクションがない設定ファイル
        let toml_content = r#"
[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_run(temp_dir.path(), &[]);
        
        // command.runセクションが見つからないため、エラーが返される
        assert!(result.is_err());
    }

    #[test]
    fn test_process_run_with_valid_config_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.runセクションがある設定ファイル
        let toml_content = r#"
[command.run]
command = "echo"
args = ["hello"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // 設定が正しく読み込まれることを確認
        // 実際のコマンド実行は環境に依存するため、設定の読み込みが成功することを確認
        let result = process_run(temp_dir.path(), &[]);
        // コマンド実行は環境に依存するが、設定の読み込みは成功する
        // エラーメッセージに設定関連のエラーが含まれていないことを確認
        if let Err(e) = &result {
            let error_msg = e.to_string();
            // 設定の読み込みエラーではないことを確認
            assert!(!error_msg.contains("Failed to read config") && 
                    !error_msg.contains("Failed to parse config") &&
                    !error_msg.contains("section not found"));
        }
    }

    #[test]
    fn test_process_run_with_extra_args() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "echo"
args = ["hello"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // extra_argsを指定
        let extra_args = vec!["world".to_string(), "test".to_string()];
        
        // 関数が呼び出し可能であることを確認
        let result = process_run(temp_dir.path(), &extra_args);
        // コマンド実行は環境に依存するが、設定の読み込みは成功する
        // エラーメッセージに設定関連のエラーが含まれていないことを確認
        if let Err(e) = &result {
            let error_msg = e.to_string();
            // 設定の読み込みエラーではないことを確認
            assert!(!error_msg.contains("Failed to read config") && 
                    !error_msg.contains("Failed to parse config") &&
                    !error_msg.contains("section not found"));
        }
    }
}

