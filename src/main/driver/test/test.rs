#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::test::process_test;

    #[test]
    fn test_process_test_without_config() {
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルが存在しない場合
        let result = process_test(temp_dir.path());
        
        // command.testセクションが見つからないため、エラーが返されるはず
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("command.test") && 
                error_msg.contains("not found"));
    }

    #[test]
    fn test_process_test_with_incomplete_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.testセクションがない設定ファイル
        let toml_content = r#"
[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_test(temp_dir.path());
        
        // command.testセクションが見つからないため、エラーが返される
        assert!(result.is_err());
    }

    #[test]
    fn test_process_test_with_no_driver_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.testセクションがあるが、driver_patternsがない設定ファイル
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_test(temp_dir.path());
        
        // driver_filesが空の場合、警告は出るが成功する
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_test_with_driver_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // driver_patternsとcommand.testセクションがある設定ファイル
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
resolution = "$1/$2.$3"

[command.test]
command = "cargo"
args = ["test", "{driver_file}"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // driver_filesが空の場合、警告は出るが成功する
        // 実際のコマンド実行は環境に依存するため、設定の読み込みが正しく行われることを確認
        let result = process_test(temp_dir.path());
        // driver_filesが空の場合は成功を返す
        assert!(result.is_ok());
    }

}

