#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::run::process_run;

    #[test]
    fn test_process_run_without_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let result = process_run(&config_path, &[]);
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("Failed to read"));
    }

    #[test]
    fn test_process_run_with_incomplete_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_run(&config_path, &[]);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_process_run_with_valid_config_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "echo"
args = ["hello"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_run(&config_path, &[]);
        if let Err(e) = &result {
            let error_msg = e.to_string();
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
        
        let extra_args = vec!["world".to_string(), "test".to_string()];
        
        let result = process_run(&config_path, &extra_args);
        if let Err(e) = &result {
            let error_msg = e.to_string();
            assert!(!error_msg.contains("Failed to read config") && 
                    !error_msg.contains("Failed to parse config") &&
                    !error_msg.contains("section not found"));
        }
    }
}

