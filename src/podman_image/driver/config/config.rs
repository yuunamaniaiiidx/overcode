#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;

    /// podman_image.rsがconfigに期待する動作をテストする
    /// 
    /// podman_image::ensure_images関数は以下の動作をconfigに期待している:
    /// 1. Config::load_from_root(root_dir)がResult<Config>を返す
    /// 2. config.command.test.imageとconfig.command.run.imageからイメージを取得できる
    /// 3. イメージが指定されていない場合は空のHashSetが返される

    #[test]
    fn test_config_load_from_root_returns_result() {
        // Config::load_from_rootがResult<Config>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let result = Config::load_from_root(temp_dir.path());
        
        // Resultが返されることを確認
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_command_test_image_exists() {
        // command.test.imageが存在することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
image = "docker.io/library/ubuntu:latest"
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // command.test.imageが存在することを確認
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert_eq!(command.test.unwrap().image, Some("docker.io/library/ubuntu:latest".to_string()));
    }

    #[test]
    fn test_config_command_run_image_exists() {
        // command.run.imageが存在することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["run"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // command.run.imageが存在することを確認
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.run.is_some());
        assert_eq!(command.run.unwrap().image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_no_images_when_command_missing() {
        // commandセクションがない場合、イメージが取得できないことを確認
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルが存在しない場合、commandはNone
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        assert!(config.command.is_none());
        
        // 明示的にcommandセクションがない場合
        let config_path = temp_dir.path().join("overcode.toml");
        let toml_content = r#"
# commandセクションなし
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        assert!(config.command.is_none());
    }

    #[test]
    fn test_config_both_test_and_run_images() {
        // command.test.imageとcommand.run.imageの両方が存在する場合
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
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
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // 両方のイメージが存在することを確認
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert!(command.run.is_some());
        assert_eq!(command.test.unwrap().image, Some("docker.io/library/ubuntu:latest".to_string()));
        assert_eq!(command.run.unwrap().image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_duplicate_images_handled() {
        // command.test.imageとcommand.run.imageが同じ場合、重複削除されることを確認
        // （実際の重複削除はpodman_image.rsで行われる）
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["test"]

[command.run]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["run"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // 両方のイメージが同じであることを確認
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        let test_image = command.test.unwrap().image;
        let run_image = command.run.unwrap().image;
        assert_eq!(test_image, run_image);
        assert_eq!(test_image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_image_optional() {
        // imageフィールドがオプショナルであることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // imageがNoneであることを確認
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert_eq!(command.test.unwrap().image, None);
    }
}
