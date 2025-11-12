use anyhow::Context;
use filetime::{set_file_mtime, FileTime};
use ignore::WalkBuilder;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;
use crate::config::Config;
use crate::podman_mount;
use log::{info, warn};

/// driver_patternsのパターンにマッチしたファイルを取得する
/// 元のファイル名（変換前）を返すことで、{src_name}.driver.{testtitle}.rs形式のファイルを個別に実行できる
fn find_driver_matched_files(config: &Config, root_dir: &Path) -> anyhow::Result<Vec<String>> {
    // WalkBuilderを構築
    let mut builder = WalkBuilder::new(root_dir);
    builder
        .hidden(false)
        .git_ignore(false)  // デフォルトの.gitignoreは無効化
        .git_exclude(true);
    
    let walker = builder.build();
    
    // 各driver_patternパターンをコンパイル
    let mut compiled_patterns = Vec::new();
    for mapping in &config.driver_patterns {
        let pattern = Regex::new(&mapping.pattern)
            .with_context(|| format!("Invalid regex pattern: {}", mapping.pattern))?;
        compiled_patterns.push(pattern);
    }
    
    let mut matched_files = Vec::new();
    
    // ファイルシステムをスキャン
    for result in walker {
        let entry = result?;
        let path = entry.path();
        
        // ファイルのみを処理
        if !path.is_file() {
            continue;
        }
        
        // 相対パスを取得
        let relative_path = path.strip_prefix(root_dir)?
            .to_string_lossy()
            .to_string();
        
        // driver_patternsにマッチするかチェック
        for pattern in &compiled_patterns {
            if pattern.is_match(&relative_path) {
                matched_files.push(relative_path.clone());
                break;  // 1つのパターンにマッチすれば十分
            }
        }
    }
    
    // 重複を除去してソート
    matched_files.sort();
    matched_files.dedup();
    
    Ok(matched_files)
}

/// mock_patternsのパターンにマッチしたファイルを取得する
fn find_mock_matched_files(config: &Config, root_dir: &Path) -> anyhow::Result<Vec<String>> {
    // WalkBuilderを構築
    let mut builder = WalkBuilder::new(root_dir);
    builder
        .hidden(false)
        .git_ignore(false)  // デフォルトの.gitignoreは無効化
        .git_exclude(true);
    
    let walker = builder.build();
    
    // 各mock_patternパターンをコンパイル
    let mut compiled_patterns = Vec::new();
    for mapping in &config.mock_patterns {
        let pattern = Regex::new(&mapping.pattern)
            .with_context(|| format!("Invalid regex pattern: {}", mapping.pattern))?;
        compiled_patterns.push(pattern);
    }
    
    let mut matched_files = Vec::new();
    
    // ファイルシステムをスキャン
    for result in walker {
        let entry = result?;
        let path = entry.path();
        
        // ファイルのみを処理
        if !path.is_file() {
            continue;
        }
        
        // 相対パスを取得
        let relative_path = path.strip_prefix(root_dir)?
            .to_string_lossy()
            .to_string();
        
        // mock_patternsにマッチするかチェック
        for pattern in &compiled_patterns {
            if pattern.is_match(&relative_path) {
                matched_files.push(relative_path.clone());
                break;  // 1つのパターンにマッチすれば十分
            }
        }
    }
    
    // 重複を除去してソート
    matched_files.sort();
    matched_files.dedup();
    
    Ok(matched_files)
}

/// パターンにマッチしたファイルパスをtestcaseで解決する
fn resolve_testcase(file_path: &str, pattern: &Regex, testcase: &str) -> Option<String> {
    if let Some(captures) = pattern.captures(file_path) {
        let mut resolved = testcase.to_string();
        // $1, $2, ... をキャプチャグループの値に置換
        for i in 1..=captures.len() - 1 {
            if let Some(capture) = captures.get(i) {
                let placeholder = format!("${}", i);
                resolved = resolved.replace(&placeholder, capture.as_str());
            }
        }
        Some(resolved)
    } else {
        None
    }
}

fn refresh_mock_mtime(path: &Path) -> anyhow::Result<()> {
    let file_time = FileTime::from_system_time(SystemTime::now());
    set_file_mtime(path, file_time)
        .with_context(|| format!("Failed to update mtime for mock file: {}", path.display()))?;
    Ok(())
}

fn restore_mock_mtime(backups: &[(PathBuf, FileTime)]) -> anyhow::Result<()> {
    for (path, original_time) in backups {
        set_file_mtime(path, *original_time).with_context(|| {
            format!(
                "Failed to restore original mtime for mock file: {}",
                path.display()
            )
        })?;
    }
    Ok(())
}

/// テストコマンドを実行する
fn execute_test_command(
    run_test: &crate::config::RunTestConfig,
    driver_file: &str,
    root_dir: &Path,
    mount_args: &[String],
) -> anyhow::Result<()> {
    let root_dir_str = root_dir.display().to_string();
    
    // replace_ruleを適用する前のdriver_fileをログ出力
    info!("Before replace_rule application: driver_file = '{}'", driver_file);
    
    let mut processed_driver_file = driver_file.to_string();

    for rule in &run_test.replace_rule {
        info!("Applying replace_rule: pattern = '{}', replace = '{}'", rule.pattern, rule.replace);
    
        let re = Regex::new(&rule.pattern).unwrap();
        let replaced = re.replace(processed_driver_file.as_str(), |caps: &regex::Captures| {
            // rule.replace が "$1::driver_$2_$3" の場合を想定
            // クロージャ内で明示的に置換
            rule.replace
                .replace("$1", &caps[1])
                .replace("$2", &caps[2])
                .replace("$3", &caps[3])
        });
    
        processed_driver_file = replaced.to_string();
        info!("After replace_rule application: '{}' -> '{}'", driver_file, processed_driver_file);
    }
    
    // args内の{driver_file}、{root_dir}を置換
    let processed_args: Vec<String> = run_test.args
        .iter()
        .map(|arg| {
            arg.replace("{driver_file}", &processed_driver_file)
               .replace("{root_dir}", &root_dir_str)
        })
        .collect();
    
    // podman runでコンテナ内で実行（必須）
    let image = run_test.image
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("image is required in [command.test] section"))?;
    
    info!("Executing in podman container (image: {}): {} {:?}", image, run_test.command, processed_args);
    
    // podman runコマンドを構築
    let root_dir_str = root_dir.display().to_string();
    let mut podman_args = vec![
        "run".to_string(),
        "--rm".to_string(),
    ];
    podman_args.extend_from_slice(mount_args);
    podman_args.push("-w".to_string());
    podman_args.push(root_dir_str);
    podman_args.push(image.clone());
    podman_args.push(run_test.command.clone());
    podman_args.extend(processed_args);
    
    let output = Command::new("podman")
        .args(&podman_args)
        .output()
        .with_context(|| format!("Failed to execute podman run for image: {}", image))?;
    
    // 標準出力と標準エラー出力をそのまま出力
    std::io::stdout().write_all(&output.stdout)
        .context("Failed to write stdout")?;
    std::io::stderr().write_all(&output.stderr)
        .context("Failed to write stderr")?;
    
    if !output.status.success() {
        anyhow::bail!(
            "Test command failed with exit code: {:?}",
            output.status.code()
        );
    }
    
    Ok(())
}

/// テスト処理を実行する
pub fn process_test(config_path: &Path) -> anyhow::Result<()> {
    let config = Config::load(config_path)?;
    let root_dir = config_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Config file has no parent directory"))?;
    
    // mock_patternsでマッチしたファイルを取得し、解決済みキーでHashMapに保存
    let mock_files = find_mock_matched_files(&config, root_dir)?;
    let mut mock_map: HashMap<String, Vec<String>> = HashMap::new();
    
    // 各mock_patternパターンをコンパイル
    let mut mock_patterns_compiled = Vec::new();
    for mapping in &config.mock_patterns {
        let pattern = Regex::new(&mapping.pattern)
            .with_context(|| format!("Invalid regex pattern: {}", mapping.pattern))?;
        mock_patterns_compiled.push((pattern, &mapping.testcase, mapping.mount_path.as_deref()));
    }
    
    // モックファイルを解決してHashMapに保存（パターン情報も保持）
    let mut mock_file_info: Vec<(String, String, Option<&str>)> = Vec::new(); // (mock_file, resolved_key, mount_path)
    for mock_file in &mock_files {
        for (pattern, testcase, mount_path) in &mock_patterns_compiled {
            if let Some(resolved_key) = resolve_testcase(mock_file, pattern, testcase) {
                mock_map.entry(resolved_key.clone()).or_insert_with(Vec::new).push(mock_file.clone());
                mock_file_info.push((mock_file.clone(), resolved_key, *mount_path));
                break;  // 1つのパターンにマッチすれば十分
            }
        }
    }
    
    // driver_patternsでマッチしたファイルを取得
    let driver_files = find_driver_matched_files(&config, root_dir)?;
    
    let run_test = config.command
        .as_ref()
        .and_then(|c| c.test.as_ref())
        .ok_or_else(|| anyhow::anyhow!("[command.test] section not found in overcode.toml"))?;
    
    if driver_files.is_empty() {
        warn!("No files matched driver_patterns pattern. Nothing to test.");
        return Ok(());
    }
    
    info!("Found {} driver file(s) to test", driver_files.len());
    
    // 各driver_patternパターンをコンパイル
    let mut driver_patterns_compiled = Vec::new();
    for mapping in &config.driver_patterns {
        let pattern = Regex::new(&mapping.pattern)
            .with_context(|| format!("Invalid regex pattern: {}", mapping.pattern))?;
        driver_patterns_compiled.push((pattern, &mapping.testcase));
    }
    
    // 各ファイルに対して一つずつ実行
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for driver_file in &driver_files {
        info!("Testing driver file: {}", driver_file);
        
        // driver_fileをtestcaseで解決
        let mut driver_resolved_key: Option<String> = None;
        for (pattern, testcase) in &driver_patterns_compiled {
            if let Some(resolved) = resolve_testcase(driver_file, pattern, testcase) {
                driver_resolved_key = Some(resolved);
                break;  // 1つのパターンにマッチすれば十分
            }
        }
        
        // ベースのマウント情報を作成
        let mut mount_args = podman_mount::build_mount_args(root_dir);
        let mut mock_mtime_backups: Vec<(PathBuf, FileTime)> = Vec::new();
        
        // 解決キーがHashMapに存在する場合、対応するモックファイルを読み込み専用マウントで追加
        if let Some(ref resolved_key) = driver_resolved_key {
            if let Some(mock_paths) = mock_map.get(resolved_key) {
                for mock_path in mock_paths {
                    // このモックファイルに対応するmount_pathを取得
                    let mount_path_template = mock_file_info.iter()
                        .find(|(file, key, _)| file == mock_path && key == resolved_key)
                        .and_then(|(_, _, mount_path)| *mount_path)
                        .ok_or_else(|| anyhow::anyhow!(
                            "mount_path is required for mock file: {} (matched pattern in mock_patterns)",
                            mock_path
                        ))?;
                    
                    // mount_pathテンプレートを解決（$1, $2, ...をキャプチャグループの値に置換）
                    let pattern = mock_patterns_compiled.iter()
                        .find(|(p, _, _)| p.is_match(mock_path))
                        .map(|(p, _, _)| p)
                        .ok_or_else(|| anyhow::anyhow!(
                            "Failed to find matching pattern for mock file: {}",
                            mock_path
                        ))?;
                    
                    let captures = pattern.captures(mock_path)
                        .ok_or_else(|| anyhow::anyhow!(
                            "Failed to capture groups from mock file path: {} with pattern",
                            mock_path
                        ))?;
                    
                    let mut original_path = mount_path_template.to_string();
                    for i in 1..=captures.len() - 1 {
                        if let Some(capture) = captures.get(i) {
                            let placeholder = format!("${}", i);
                            original_path = original_path.replace(&placeholder, capture.as_str());
                        }
                    }
                    
                    let mock_abs_path = root_dir.join(mock_path);
                    let original_abs_path = root_dir.join(&original_path);

                    let metadata = fs::metadata(&mock_abs_path).with_context(|| {
                        format!(
                            "Failed to retrieve metadata for mock file: {}",
                            mock_abs_path.display()
                        )
                    })?;
                    let original_time = FileTime::from_last_modification_time(&metadata);
                    mock_mtime_backups.push((mock_abs_path.clone(), original_time));
                    refresh_mock_mtime(&mock_abs_path)?;
                    
                    // 読み込み専用マウント: -v {mock_path}:{original_path}:ro
                    mount_args.push("-v".to_string());
                    mount_args.push(format!("{}:{}:ro", 
                        mock_abs_path.display(), 
                        original_abs_path.display()));
                    
                    info!("Mounting mock file: {} -> {} (read-only)", mock_path, original_path);
                }
            }
        }
        
        let command_result = execute_test_command(
            &run_test,
            driver_file,
            root_dir,
            &mount_args,
        );

        restore_mock_mtime(&mock_mtime_backups)?;

        match command_result {
            Ok(_) => {
                info!("✓ Test passed for: {}", driver_file);
                success_count += 1;
            }
            Err(e) => {
                warn!("✗ Test failed for {}: {}", driver_file, e);
                failure_count += 1;
                // エラーを記録するが、次のファイルのテストは続行
            }
        }
    }
    
    info!("Test summary: {} passed, {} failed", success_count, failure_count);
    
    if failure_count > 0 {
        anyhow::bail!("Some tests failed: {} out of {} failed", failure_count, driver_files.len());
    }
    
    Ok(())
}

#[cfg(test)]
#[path = "test/driver/config/config.rs"]
mod driver_config_config;

#[cfg(test)]
#[path = "test/driver/podman_mount/podman_mount.rs"]
mod driver_podman_mount_podman_mount;

