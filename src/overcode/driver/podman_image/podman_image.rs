#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::podman_image::ensure_images;
    use crate::config::Config;

    #[test]
    fn test_ensure_images_with_empty_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 空の設定ファイルを作成
        let toml_content = r#"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // ensure_imagesを実行
        let result = ensure_images(&config_path);
        
        // イメージが指定されていない場合は成功する
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_images_with_command_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // イメージが指定された設定ファイルを作成
        let toml_content = r#"
[command.test]
image = "docker.io/library/ubuntu:latest"
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // ensure_imagesを実行
        // 注意: このテストは実際にpodmanコマンドを実行するため、
        // podmanがインストールされていない環境では失敗する可能性があります
        let result = ensure_images(&config_path);
        
        // 結果は環境に依存するが、設定の読み込みは成功する
        // エラーメッセージに設定関連のエラーが含まれていないことを確認
        if let Err(e) = &result {
            let error_msg = e.to_string();
            // 設定の読み込みエラーではないことを確認
            assert!(!error_msg.contains("Failed to read config") && 
                    !error_msg.contains("Failed to parse config"));
        }
    }

    #[test]
    fn test_ensure_images_loads_config_correctly() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 複数のイメージが指定された設定ファイルを作成
        let toml_content = r#"
[command.test]
image = "docker.io/library/ubuntu:latest"
command = "cargo"
args = ["test"]

[command.run]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["run"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        // 設定が正しく読み込まれることを確認
        let config = Config::load(&config_path).unwrap();
        // 新しい実装では、command.test.imageとcommand.run.imageからイメージを取得する
        // このテストは、commandセクションが正しく読み込まれることを確認する
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert!(command.run.is_some());
        assert_eq!(command.test.unwrap().image, Some("docker.io/library/ubuntu:latest".to_string()));
        assert_eq!(command.run.unwrap().image, Some("docker.io/library/rust:latest".to_string()));
    }
}

