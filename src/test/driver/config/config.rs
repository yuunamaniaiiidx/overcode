#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;

    /// test.rsがconfigに期待する動作をテストする
    /// 
    /// test::process_test関数とfind_driver_matched_files関数は以下の動作をconfigに期待している:
    /// 1. Config::load(config_path)がResult<Config>を返す
    /// 2. config.driver_patternsがVec<MappingEntry>である
    /// 3. config.driver_patternsをイテレートできる
    /// 4. mapping.patternがStringである
    /// 5. config.commandがOption<CommandConfig>である
    /// 6. config.command.as_ref().and_then(|c| c.test.as_ref())が動作する
    /// 7. run_testが&RunTestConfigとして取得できる
    /// 8. run_test.commandがStringである
    /// 9. run_test.argsがVec<String>である
    /// 10. run_test.args.iter()が動作する
    /// 11. run_test.args.iter().map(...)が動作する
    /// 12. run_test.replace_ruleがVec<ReplaceRule>である
    /// 13. run_test.replace_rule.iter()が動作する
    /// 14. rule.patternとrule.replaceがStringである
    /// 15. run_test.imageがOption<String>である
    /// 16. run_test.image.as_ref()が動作する

    #[test]
    fn test_config_load_returns_result() {
        // Config::loadがResult<Config>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        fs::write(&config_path, "").unwrap();
        let result = Config::load(&config_path);
        
        // Resultが返されることを確認
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_driver_patterns_is_vec_mapping_entry() {
        // config.driver_patternsがVec<MappingEntry>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[driver_patterns]]
pattern = "(.+)/(.+)/driver/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
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
testcase = "$1/$2.$3"

[[driver_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
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
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
        // and_thenが動作することを確認
        let test_config = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref());
        
        assert!(test_config.is_some());
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
        
        let config = Config::load(&config_path).unwrap();
        
        // test.rsの実際の使用パターン: command.testを使用
        let run_test: &crate::config::RunTestConfig = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        
        let config = Config::load(&config_path).unwrap();
        
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
        // test.rsの実際の使用パターン: command.testのみを使用
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.testが指定されている場合
        let toml_content = r#"
[command.test]
command = "cargo"
args = ["test", "command_test"]
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        // command.testが使用されることを確認
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref())
            .expect("command.test should exist");
        
        assert_eq!(run_test.args[1], "command_test");
    }


    #[test]
    fn test_config_no_test_config_error_case() {
        // test.rsの実際の使用パターン: command.testがない場合
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // command.testがない設定ファイルを作成
        let toml_content = r#"
# command.testがない
"#;
        fs::write(&config_path, toml_content).unwrap();
        let config = Config::load(&config_path).unwrap();
        
        // command.testがないことを確認
        let run_test = config.command
            .as_ref()
            .and_then(|c| c.test.as_ref());
        
        assert!(run_test.is_none());
    }

    #[test]
    fn test_config_mock_patterns_is_vec_mapping_entry() {
        // config.mock_patternsがVec<MappingEntry>であることを確認
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
        
        // mock_patternsがVec<MappingEntry>であることを確認
        assert_eq!(config.mock_patterns.len(), 1);
    }

    #[test]
    fn test_config_mock_patterns_can_be_iterated() {
        // config.mock_patternsをイテレートできることを確認
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
        
        // イテレートできることを確認
        let mut count = 0;
        for _mapping in &config.mock_patterns {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_mock_patterns_mapping_testcase_is_string() {
        // mapping.testcaseがStringであることを確認
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
        
        // testcaseがStringであることを確認
        assert_eq!(config.mock_patterns[0].testcase, "$1/$2.$3");
        let _testcase_str: &str = &config.mock_patterns[0].testcase;
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_is_option_string() {
        // mapping.mount_pathがOption<String>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // mount_pathが指定されている場合
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
mount_path = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        // mount_pathがOption<String>であることを確認
        assert!(config.mock_patterns[0].mount_path.is_some());
        assert_eq!(config.mock_patterns[0].mount_path.as_ref().unwrap(), "$1/$2.$3");
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_none_case() {
        // mapping.mount_pathがNoneの場合の動作を確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        // mount_pathが指定されていない場合
        let toml_content = r#"
[[mock_patterns]]
pattern = "(.+)/(.+)/mock/.+.(.+)"
testcase = "$1/$2.$3"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        
        // mount_pathがNoneであることを確認
        assert!(config.mock_patterns[0].mount_path.is_none());
    }

    #[test]
    fn test_mock_patterns_mapping_mount_path_as_deref_works() {
        // mapping.mount_path.as_deref()が動作することを確認（test.rsの実際の使用パターン）
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
        
        // as_deref()が動作することを確認（test.rsの実際の使用パターンを再現）
        for mapping in &config.mock_patterns {
            let mount_path_opt: Option<&str> = mapping.mount_path.as_deref();
            assert!(mount_path_opt.is_some());
            assert_eq!(mount_path_opt.unwrap(), "$1/$2.$3");
        }
    }

    #[test]
    fn test_mock_patterns_compilation_pattern() {
        // test.rsの実際の使用パターン: mock_patternsのコンパイル
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
        
        // test.rsと同じパターンでmock_patternsをコンパイルできることを確認
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

