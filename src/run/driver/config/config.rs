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
    fn test_config_command_is_option_command_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_some());
    }

    #[test]
    fn test_config_command_as_ref_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let command_ref = config.command.as_ref();
        assert!(command_ref.is_some());
    }

    #[test]
    fn test_config_command_and_then_run_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_some());
    }

    #[test]
    fn test_config_command_run_is_option_run_test_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        if let Some(command_config) = config.command.as_ref() {
            assert!(command_config.run.is_some());
        } else {
            panic!("command should be Some");
        }
    }

    #[test]
    fn test_run_config_can_be_obtained_as_ref() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config: &crate::config::RunTestConfig = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert_eq!(run_config.command, "cargo");
    }

    #[test]
    fn test_run_config_command_is_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert_eq!(run_config.command, "cargo");
        let _command_str: &str = &run_config.command;
    }

    #[test]
    fn test_run_config_args_is_vec_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "--manifest-path", "Cargo.toml"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert_eq!(run_config.args.len(), 3);
        assert_eq!(run_config.args[0], "test");
        assert_eq!(run_config.args[1], "--manifest-path");
        assert_eq!(run_config.args[2], "Cargo.toml");
    }

    #[test]
    fn test_run_config_args_iter_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "build"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        let mut count = 0;
        for _arg in run_config.args.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_run_config_args_iter_map_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "{root_dir}"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        let root_dir_str = temp_dir.path().display().to_string();
        let processed_args: Vec<String> = run_config.args
            .iter()
            .map(|arg| arg.replace("{root_dir}", &root_dir_str))
            .collect();
        
        assert_eq!(processed_args.len(), 2);
        assert_eq!(processed_args[0], "test");
        assert_eq!(processed_args[1], root_dir_str);
    }

    #[test]
    fn test_run_config_image_is_option_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert!(run_config.image.is_some());
        assert_eq!(run_config.image.as_ref().unwrap(), "docker.io/library/rust:latest");
    }

    #[test]
    fn test_run_config_image_as_ref_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        if let Some(ref image) = run_config.image {
            assert_eq!(image, "docker.io/library/rust:latest");
        } else {
            panic!("image should be Some");
        }
    }

    #[test]
    fn test_run_config_image_none_case() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert!(run_config.image.is_none());
    }


    #[test]
    fn test_config_command_none_case() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
# commandセクションなし
"#;
        fs::write(&config_path, toml_content).unwrap();
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.command.is_none());
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_none());
    }

    #[test]
    fn test_config_command_run_none_case() {
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
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_none());
    }
}

