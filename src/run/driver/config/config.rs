#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;

    /// run.rsがconfigに期待する動作をテストする
    /// 
    /// run::process_run関数は以下の動作をconfigに期待している:
    /// 1. Config::load_from_root(root_dir)がResult<Config>を返す
    /// 2. config.commandがOption<CommandConfig>である
    /// 3. config.command.as_ref()が動作する
    /// 4. config.command.as_ref().and_then(|c| c.run.as_ref())が動作する
    /// 5. c.runがOption<RunTestConfig>である
    /// 6. run_configが&RunTestConfigとして取得できる
    /// 7. run_config.commandがStringである
    /// 8. run_config.argsがVec<String>である
    /// 9. run_config.args.iter()が動作する
    /// 10. run_config.args.iter().map(...)が動作する
    /// 11. run_config.imageがOption<String>である
    /// 12. run_config.image.as_ref()が動作する

    #[test]
    fn test_config_load_from_root_returns_result() {
        // Config::load_from_rootがResult<Config>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let result = Config::load_from_root(temp_dir.path());
        
        // Resultが返されることを確認
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_command_is_option_command_config() {
        // config.commandがOption<CommandConfig>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // commandがOption<CommandConfig>であることを確認
        assert!(config.command.is_some());
    }

    #[test]
    fn test_config_command_as_ref_works() {
        // config.command.as_ref()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // as_ref()が動作することを確認
        let command_ref = config.command.as_ref();
        assert!(command_ref.is_some());
    }

    #[test]
    fn test_config_command_and_then_run_works() {
        // config.command.as_ref().and_then(|c| c.run.as_ref())が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // and_thenが動作することを確認
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_some());
    }

    #[test]
    fn test_config_command_run_is_option_run_test_config() {
        // c.runがOption<RunTestConfig>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // runがOption<RunTestConfig>であることを確認
        if let Some(command_config) = config.command.as_ref() {
            assert!(command_config.run.is_some());
        } else {
            panic!("command should be Some");
        }
    }

    #[test]
    fn test_run_config_can_be_obtained_as_ref() {
        // run_configが&RunTestConfigとして取得できることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // &RunTestConfigとして取得できることを確認
        let run_config: &crate::config::RunTestConfig = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        assert_eq!(run_config.command, "cargo");
    }

    #[test]
    fn test_run_config_command_is_string() {
        // run_config.commandがStringであることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // commandがStringであることを確認
        assert_eq!(run_config.command, "cargo");
        let _command_str: &str = &run_config.command;
    }

    #[test]
    fn test_run_config_args_is_vec_string() {
        // run_config.argsがVec<String>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "--manifest-path", "Cargo.toml"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // argsがVec<String>であることを確認
        assert_eq!(run_config.args.len(), 3);
        assert_eq!(run_config.args[0], "test");
        assert_eq!(run_config.args[1], "--manifest-path");
        assert_eq!(run_config.args[2], "Cargo.toml");
    }

    #[test]
    fn test_run_config_args_iter_works() {
        // run_config.args.iter()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "build"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // iter()が動作することを確認
        let mut count = 0;
        for _arg in run_config.args.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_run_config_args_iter_map_works() {
        // run_config.args.iter().map(...)が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test", "{root_dir}"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // iter().map()が動作することを確認（run.rsの実際の使用パターンを再現）
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
        // run_config.imageがOption<String>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // imageが指定されている場合
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // imageがOption<String>であることを確認
        assert!(run_config.image.is_some());
        assert_eq!(run_config.image.as_ref().unwrap(), "docker.io/library/rust:latest");
    }

    #[test]
    fn test_run_config_image_as_ref_works() {
        // run_config.image.as_ref()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // as_ref()が動作することを確認（run.rsの実際の使用パターンを再現）
        if let Some(ref image) = run_config.image {
            assert_eq!(image, "docker.io/library/rust:latest");
        } else {
            panic!("image should be Some");
        }
    }

    #[test]
    fn test_run_config_image_none_case() {
        // run_config.imageがNoneの場合の動作を確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // imageが指定されていない場合
        let toml_content = r#"
[command.run]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref())
            .expect("run config should exist");
        
        // imageがNoneであることを確認
        assert!(run_config.image.is_none());
    }


    #[test]
    fn test_config_command_none_case() {
        // config.commandがNoneの場合の動作を確認
        let temp_dir = TempDir::new().unwrap();
        
        // commandセクションがない場合
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // commandがNoneであることを確認
        assert!(config.command.is_none());
        
        // run.rsの実際の使用パターン: commandがNoneの場合、エラーになる
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_none());
    }

    #[test]
    fn test_config_command_run_none_case() {
        // config.command.runがNoneの場合の動作を確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.runが指定されていない場合
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // commandはSomeだが、runはNoneであることを確認
        assert!(config.command.is_some());
        
        let run_config = config.command
            .as_ref()
            .and_then(|c| c.run.as_ref());
        
        assert!(run_config.is_none());
    }
}

