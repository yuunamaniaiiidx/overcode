#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;

    /// test.rsがconfigに期待する動作をテストする
    /// 
    /// test::process_test関数とfind_driver_matched_files関数は以下の動作をconfigに期待している:
    /// 1. Config::load_from_root(root_dir)がResult<Config>を返す
    /// 2. config.get_ignore_patterns()がVec<IgnorePattern>を返す
    /// 3. config.get_ignore_files()がVec<String>を返す
    /// 4. config.driver_patternsがVec<MappingEntry>である
    /// 5. config.driver_patternsをイテレートできる
    /// 6. mapping.patternがStringである
    /// 7. config.commandがOption<CommandConfig>である
    /// 8. config.command.as_ref().and_then(|c| c.test.as_ref())が動作する
    /// 9. config.run_testがOption<RunTestConfig>である
    /// 10. config.run_test.as_ref()が動作する
    /// 11. run_testが&RunTestConfigとして取得できる
    /// 12. run_test.commandがStringである
    /// 13. run_test.argsがVec<String>である
    /// 14. run_test.args.iter()が動作する
    /// 15. run_test.args.iter().map(...)が動作する
    /// 16. run_test.replace_ruleがVec<ReplaceRule>である
    /// 17. run_test.replace_rule.iter()が動作する
    /// 18. rule.patternとrule.replaceがStringである
    /// 19. run_test.imageがOption<String>である
    /// 20. run_test.image.as_ref()が動作する

    #[test]
    fn test_config_load_from_root_returns_result() {
        // Config::load_from_rootがResult<Config>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let result = Config::load_from_root(temp_dir.path());
        
        // Resultが返されることを確認
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_get_ignore_patterns_returns_vec_ignore_pattern() {
        // config.get_ignore_patterns()がVec<IgnorePattern>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[ignores]]
path = ".git"

[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // get_ignore_patterns()がVec<IgnorePattern>を返すことを確認
        let ignore_patterns = config.get_ignore_patterns();
        assert_eq!(ignore_patterns.len(), 1); // pathが指定されたもののみ
    }

    #[test]
    fn test_config_get_ignore_files_returns_vec_string() {
        // config.get_ignore_files()がVec<String>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[ignores]]
path = ".git"

[[ignores]]
file = ".gitignore"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // get_ignore_files()がVec<String>を返すことを確認
        let ignore_files = config.get_ignore_files();
        assert_eq!(ignore_files.len(), 1);
        assert_eq!(ignore_files[0], ".gitignore");
    }

    #[test]
    fn test_config_driver_patterns_is_vec_mapping_entry() {
        // config.driver_patternsがVec<MappingEntry>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
resolution = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // driver_patternsがVec<MappingEntry>であることを確認
        assert_eq!(config.driver_patterns.len(), 1);
    }

    #[test]
    fn test_config_driver_patterns_can_be_iterated() {
        // config.driver_patternsをイテレートできることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
resolution = "$1/$2.$3"

[[driver_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
resolution = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // イテレートできることを確認
        let mut count = 0;
        for _mapping in &config.driver_patterns {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_mapping_pattern_is_string() {
        // mapping.patternがStringであることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
resolution = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // patternがStringであることを確認
        assert_eq!(config.driver_patterns[0].pattern, "(.+)/(.+)/driver/.+.(.+)");
        let _pattern_str: &str = &config.driver_patterns[0].pattern;
    }

    #[test]
    fn test_config_command_is_option_command_config() {
        // config.commandがOption<CommandConfig>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // commandがOption<CommandConfig>であることを確認
        assert!(config.command.is_some());
    }

    #[test]
    fn test_config_command_and_then_test_works() {
        // config.command.as_ref().and_then(|c| c.test.as_ref())が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // and_thenが動作することを確認
        let test_config = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref());
        
        assert!(test_config.is_some());
    }

    #[test]
    fn test_config_run_test_is_option_run_test_config() {
        // config.run_testがOption<RunTestConfig>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[run_test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // run_testがOption<RunTestConfig>であることを確認
        assert!(config.run_test.is_some());
    }

    #[test]
    fn test_config_run_test_as_ref_works() {
        // config.run_test.as_ref()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[run_test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // as_ref()が動作することを確認
        let run_test_ref = config.run_test.as_ref();
        assert!(run_test_ref.is_some());
    }

    #[test]
    fn test_run_test_can_be_obtained_as_ref() {
        // run_testが&RunTestConfigとして取得できることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // test.rsの実際の使用パターン: command.testを優先し、なければrun_testを使用
        let run_test: &crate::config::RunTestConfig = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .or_else(|| config.run_test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.command, "cargo");
    }

    #[test]
    fn test_run_test_command_is_string() {
        // run_test.commandがStringであることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // commandがStringであることを確認
        assert_eq!(run_test.command, "cargo");
        let _command_str: &str = &run_test.command;
    }

    #[test]
    fn test_run_test_args_is_vec_string() {
        // run_test.argsがVec<String>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "--manifest-path", "Cargo.toml"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // argsがVec<String>であることを確認
        assert_eq!(run_test.args.len(), 3);
        assert_eq!(run_test.args[0], "test");
        assert_eq!(run_test.args[1], "--manifest-path");
        assert_eq!(run_test.args[2], "Cargo.toml");
    }

    #[test]
    fn test_run_test_args_iter_works() {
        // run_test.args.iter()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "build"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // iter()が動作することを確認
        let mut count = 0;
        for _arg in run_test.args.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_run_test_args_iter_map_works() {
        // run_test.args.iter().map(...)が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "{driver_file}", "{root_dir}"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // iter().map()が動作することを確認（test.rsの実際の使用パターンを再現）
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
        // run_test.replace_ruleがVec<ReplaceRule>であることを確認
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
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // replace_ruleがVec<ReplaceRule>であることを確認
        assert_eq!(run_test.replace_rule.len(), 1);
    }

    #[test]
    fn test_run_test_replace_rule_iter_works() {
        // run_test.replace_rule.iter()が動作することを確認
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
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // iter()が動作することを確認
        let mut count = 0;
        for _rule in run_test.replace_rule.iter() {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_replace_rule_pattern_and_replace_are_string() {
        // rule.patternとrule.replaceがStringであることを確認
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
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // patternとreplaceがStringであることを確認
        assert_eq!(run_test.replace_rule[0].pattern, "(.+)/(.+)/driver/.+.(.+)");
        assert_eq!(run_test.replace_rule[0].replace, "$2");
        let _pattern_str: &str = &run_test.replace_rule[0].pattern;
        let _replace_str: &str = &run_test.replace_rule[0].replace;
    }

    #[test]
    fn test_run_test_image_is_option_string() {
        // run_test.imageがOption<String>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // imageが指定されている場合
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // imageがOption<String>であることを確認
        assert!(run_test.image.is_some());
        assert_eq!(run_test.image.as_ref().unwrap(), "docker.io/library/rust:latest");
    }

    #[test]
    fn test_run_test_image_as_ref_works() {
        // run_test.image.as_ref()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
image = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // as_ref()が動作することを確認（test.rsの実際の使用パターンを再現）
        if let Some(ref image) = run_test.image {
            assert_eq!(image, "docker.io/library/rust:latest");
        } else {
            panic!("image should be Some");
        }
    }

    #[test]
    fn test_run_test_image_none_case() {
        // run_test.imageがNoneの場合の動作を確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // imageが指定されていない場合
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // imageがNoneであることを確認
        assert!(run_test.image.is_none());
    }

    #[test]
    fn test_run_test_with_replace_rule_application() {
        // test.rsの実際の使用パターン: replace_ruleの適用
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
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("run_test should exist");
        
        // test.rsと同じパターンでreplace_ruleを適用できることを確認
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
        // test.rsの実際の使用パターン: command.testを優先し、なければrun_testを使用
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // 両方指定されている場合、command.testが優先される
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "command_test"]

[run_test]
command = "cargo"
args = ["test", "run_test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // command.testが優先されることを確認
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .or_else(|| config.run_test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.args[1], "command_test");
    }

    #[test]
    fn test_config_run_test_fallback() {
        // test.rsの実際の使用パターン: command.testがなければrun_testを使用
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.testがなく、run_testのみ指定されている場合
        let toml_content = r#"
[run_test]
command = "cargo"
args = ["test", "run_test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // run_testが使用されることを確認
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .or_else(|| config.run_test.as_ref())
            .expect("run_test should exist");
        
        assert_eq!(run_test.args[1], "run_test");
    }

    #[test]
    fn test_config_no_test_config_error_case() {
        // test.rsの実際の使用パターン: command.testもrun_testもない場合
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルが存在しない場合
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // command.testもrun_testもないことを確認
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .or_else(|| config.run_test.as_ref());
        
        assert!(run_test.is_none());
    }
}

