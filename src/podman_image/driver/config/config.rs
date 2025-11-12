#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;


    #[test]
    fn test_config_load_returns_result() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        fs::write(&config_path, "").unwrap();
        let result = Config::load(&config_path);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_command_test_image_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
image = "docker.io/library/ubuntu:latest"
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert_eq!(command.test.unwrap().image, Some("docker.io/library/ubuntu:latest".to_string()));
    }

    #[test]
    fn test_config_command_run_image_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["run"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.run.is_some());
        assert_eq!(command.run.unwrap().image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_no_images_when_command_missing() {
        let temp_dir = TempDir::new().unwrap();
        
        let config_path = temp_dir.path().join("overcode.toml");
        let toml_content = r#"
# commandセクションなし
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        assert!(config.command.is_none());
    }

    #[test]
    fn test_config_both_test_and_run_images() {
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
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert!(command.run.is_some());
        assert_eq!(command.test.unwrap().image, Some("docker.io/library/ubuntu:latest".to_string()));
        assert_eq!(command.run.unwrap().image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_duplicate_images_handled() {
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
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        let test_image = command.test.unwrap().image;
        let run_image = command.run.unwrap().image;
        assert_eq!(test_image, run_image);
        assert_eq!(test_image, Some("docker.io/library/rust:latest".to_string()));
    }

    #[test]
    fn test_config_image_optional() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
        let command = config.command.unwrap();
        assert!(command.test.is_some());
        assert_eq!(command.test.unwrap().image, None);
    }
}
