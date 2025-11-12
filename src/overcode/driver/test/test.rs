#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::test::process_test;

    #[test]
    fn test_process_test_without_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let result = process_test(&config_path);
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("Failed to read"));
    }

    #[test]
    fn test_process_test_with_incomplete_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_test(&config_path);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_process_test_with_no_driver_files() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_test(&config_path);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_test_with_driver_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
testcase = "$1/$2.$3"

[command.test]
command = "cargo"
args = ["test", "{driver_file}"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let result = process_test(&config_path);
        assert!(result.is_ok());
    }

}

