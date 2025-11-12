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
    fn test_config_driver_patterns_is_vec_mapping_entry() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert_eq!(config.driver_patterns.len(), 1);
    }

    #[test]
    fn test_config_driver_patterns_can_be_iterated() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
testcase = "$1/$2.$3"

[[driver_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let mut count = 0;
        for _mapping in &config.driver_patterns {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_mapping_pattern_is_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert_eq!(config.driver_patterns[0].pattern, "(.+)/(.+)/driver/.+.(.+)");
        let _pattern_str: &str = &config.driver_patterns[0].pattern;
    }

    #[test]
    fn test_config_command_is_option_command_config() {
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
    }

    #[test]
    fn test_config_command_and_then_test_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let test_config = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref());
        
        assert!(test_config.is_some());
    }

    #[test]
    fn test_run_test_can_be_obtained_as_ref() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test: &crate::config::RunTestConfig = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.command, "cargo");
    }

    #[test]
    fn test_run_test_command_is_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.command, "cargo");
        let _command_str: &str = &run_test.command;
    }

    #[test]
    fn test_run_test_args_is_vec_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "--manifest-path", "Cargo.toml"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.args.len(), 3);
        assert_eq!(run_test.args[0], "test");
        assert_eq!(run_test.args[1], "--manifest-path");
        assert_eq!(run_test.args[2], "Cargo.toml");
    }

    #[test]
    fn test_run_test_args_iter_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "build"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        let mut count = 0;
        for _arg in run_test.args.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_run_test_args_iter_map_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "{driver_file}", "{root_dir}"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        let root_dir_str = temp_dir.path().display().to_string();
        let driver_file = "test.rs";
        let processed_args: Vec<String> = run_test.args
            .iter()
            .map(|arg| {
                arg.replace("{driver_file}", driver_file)
                   .replace("{root_dir}", &root_dir_str)
            })
            .collect();
        
        assert_eq!(processed_args.len(), 3);
        assert_eq!(processed_args[0], "test");
        assert_eq!(processed_args[1], driver_file);
        assert_eq!(processed_args[2], root_dir_str);
    }

    #[test]
    fn test_run_test_replace_rule_is_vec_replace_rule() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
replace_rule = [
    { pattern = "(.+)/(.+)/driver/.+.(.+)", replace = "$2" }
]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.replace_rule.len(), 1);
    }

    #[test]
    fn test_run_test_replace_rule_iter_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
replace_rule = [
    { pattern = "(.+)/(.+)/driver/.+.(.+)", replace = "$2" },
    { pattern = "(.+)/(.+)/mock/.+.(.+)", replace = "$2" }
]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        let mut count = 0;
        for _rule in run_test.replace_rule.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_replace_rule_pattern_and_replace_are_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
replace_rule = [
    { pattern = "(.+)/(.+)/driver/.+.(.+)", replace = "$2" }
]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.replace_rule[0].pattern, "(.+)/(.+)/driver/.+.(.+)");
        assert_eq!(run_test.replace_rule[0].replace, "$2");
        let _pattern_str: &str = &run_test.replace_rule[0].pattern;
        let _replace_str: &str = &run_test.replace_rule[0].replace;
    }

    #[test]
    fn test_run_test_image_is_option_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert!(run_test.image.is_some());
        assert_eq!(run_test.image.as_ref().unwrap(), "docker.io/library/rust:latest");
    }

    #[test]
    fn test_run_test_image_as_ref_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        if let Some(ref image) = run_test.image {
            assert_eq!(image, "docker.io/library/rust:latest");
        } else {
            panic!("image should be Some");
        }
    }

    #[test]
    fn test_run_test_image_none_case() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        assert!(run_test.image.is_none());
    }

    #[test]
    fn test_run_test_with_replace_rule_application() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "{driver_file}"]
replace_rule = [
    { pattern = "(.+)/(.+)/driver/.+.(.+)", replace = "$2" }
]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        let driver_file = "src/test/driver/config.rs";
        let mut processed_driver_file = driver_file.to_string();
        
        for rule in &run_test.replace_rule {
            use regex::Regex;
            let regex = Regex::new(&rule.pattern).unwrap();
            processed_driver_file = regex.replace(&processed_driver_file, rule.replace.as_str()).to_string();
        }
        
        assert_eq!(processed_driver_file, "test");
    }

    #[test]
    fn test_config_command_test_priority_over_run_test() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "command_test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("command.test should exist");
        
        assert_eq!(run_test.args[1], "command_test");
    }


    #[test]
    fn test_config_no_test_config_error_case() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
# command.testがない
"#;
        fs::write(&config_path, toml_content).unwrap();
        let config = Config::load(&config_path).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref());
        
        assert!(run_test.is_none());
    }

    #[test]
    fn test_config_mock_patterns_is_vec_mapping_entry() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert_eq!(config.mock_patterns.len(), 1);
    }

    #[test]
    fn test_config_mock_patterns_can_be_iterated() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"

[[mock_patterns]]
pattern = "(.+)/(.+)/mock2/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        let mut count = 0;
        for _mapping in &config.mock_patterns {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_mock_patterns_mapping_testcase_is_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert_eq!(config.mock_patterns[0].testcase, "$1/$2.$3");
        let _testcase_str: &str = &config.mock_patterns[0].testcase;
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_is_option_string() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.mock_patterns[0].mount_path.is_some());
        assert_eq!(config.mock_patterns[0].mount_path.as_ref().unwrap(), "$1/$2.$3");
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_none_case() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        assert!(config.mock_patterns[0].mount_path.is_none());
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_as_deref_works() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        for mapping in &config.mock_patterns {
            let mount_path_opt: Option<&str> = mapping.mount_path.as_deref();
            assert!(mount_path_opt.is_some());
            assert_eq!(mount_path_opt.unwrap(), "$1/$2.$3");
        }
    }

    #[test]
    fn test_mock_patterns_compilation_pattern() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        use regex::Regex;
        let mut mock_patterns_compiled = Vec::new();
        for mapping in &config.mock_patterns {
            let pattern = Regex::new(&mapping.pattern).unwrap();
            mock_patterns_compiled.push((pattern, &mapping.testcase, mapping.mount_path.as_deref()));
        }
        
        assert_eq!(mock_patterns_compiled.len(), 1);
        let (pattern, testcase, mount_path) = &mock_patterns_compiled[0];
        assert!(pattern.is_match("src/test/mock/config.rs"));
        assert_eq!(*testcase, "$1/$2.$3");
        assert_eq!(*mount_path, Some("$1/$2.$3"));
    }
}

