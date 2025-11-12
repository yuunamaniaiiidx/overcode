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
        
        let toml_content = r#"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = ensure_images(&config_path);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_images_with_command_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
image = "docker.io/library/ubuntu:latest"
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = ensure_images(&config_path);
        
        if let Err(e) = &result {
            let error_msg = e.to_string();
            assert!(!error_msg.contains("Failed to read config") && 
                    !error_msg.contains("Failed to parse config"));
        }
    }

    #[test]
    fn test_ensure_images_loads_config_correctly() {
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
}

