#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::config::Config;

    #[test]
    fn test_config_init_config_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 設定ファイルが存在しないことを確認
        assert!(!config_path.exists());
        
        // init_configを実行
        let result = Config::init_config(temp_dir.path());
        assert!(result.is_ok());
        
        // 設定ファイルが作成されたことを確認
        assert!(config_path.exists());
        
        // ファイルの内容を確認
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# overcode.toml"));
    }

    #[test]
    fn test_config_init_config_when_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 既存の設定ファイルを作成
        fs::write(&config_path, "existing content").unwrap();
        
        // init_configを実行（既存ファイルがある場合）
        let result = Config::init_config(temp_dir.path());
        assert!(result.is_ok());
        
        // ファイルの内容が変更されていないことを確認
        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, "existing content");
    }

    #[test]
    fn test_config_load_with_existing_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 有効なTOMLファイルを作成
        let toml_content = r#"
[[driver_patterns]]
pattern = "test"
testcase = "test"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // loadを実行
        let result = Config::load(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.driver_patterns.len(), 1);
    }

    #[test]
    fn test_config_load_without_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 設定ファイルが存在しない場合、エラーが返されることを確認
        let result = Config::load(&config_path);
        assert!(result.is_err());
    }

}

