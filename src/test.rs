use anyhow::Context;
use std::path::Path;
use std::process::Command;
use std::io::Write;
use regex::Regex;
use ignore::WalkBuilder;
use crate::config::Config;
use crate::podman_mount;
use log::{info, warn};

/// driver_patternsのパターンにマッチしたファイルを取得する
/// 元のファイル名（変換前）を返すことで、{src_name}.driver.{testtitle}.rs形式のファイルを個別に実行できる
fn find_driver_matched_files(config: &Config, root_dir: &Path) -> anyhow::Result<Vec<String>> {
    let ignore_patterns = config.get_ignore_patterns();
    let ignore_files = config.get_ignore_files();
    
    // WalkBuilderを構築
    let mut builder = WalkBuilder::new(root_dir);
    builder
        .hidden(false)
        .git_ignore(false)  // デフォルトの.gitignoreは無効化
        .git_exclude(true);
    
    // 設定から読み込んだignoreファイルを追加
    for ignore_file in ignore_files {
        let ignore_path = root_dir.join(&ignore_file);
        if ignore_path.exists() {
            // WalkBuilderにignoreファイルを追加
            if let Some(err) = builder.add_ignore(&ignore_path) {
                return Err(anyhow::anyhow!("Failed to add ignore file {:?}: {}", ignore_path, err));
            }
        }
    }
    
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
        
        // ignoreパターンで除外
        let should_ignore = ignore_patterns.iter().any(|pattern| pattern.matches(path, root_dir));
        if should_ignore {
            continue;
        }
        
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

/// テストコマンドを実行する
fn execute_test_command(
    run_test: &crate::config::RunTestConfig,
    driver_file: &str,
    root_dir: &Path,
) -> anyhow::Result<()> {
    let root_dir_str = root_dir.display().to_string();
    
    // replace_ruleを適用してdriver_fileを変換
    let mut processed_driver_file = driver_file.to_string();
    for rule in &run_test.replace_rule {
        let regex = Regex::new(&rule.pattern)
            .with_context(|| format!("Invalid regex pattern in replace_rule: {}", rule.pattern))?;
        
        // 正規表現でマッチした場合、置換文字列内の$1, $2, ...を自動的にキャプチャグループの値に置換
        processed_driver_file = regex.replace(&processed_driver_file, rule.replace.as_str()).to_string();
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
        .ok_or_else(|| anyhow::anyhow!("image is required in [command.test] or [run_test] section"))?;
    
    info!("Executing in podman container (image: {}): {} {:?}", image, run_test.command, processed_args);
    
    // podman runコマンドを構築
    let root_dir_str = root_dir.display().to_string();
    let mut podman_args = vec![
        "run".to_string(),
        "--rm".to_string(),
    ];
    podman_args.extend(podman_mount::build_mount_args(root_dir));
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
pub fn process_test(root_dir: &Path) -> anyhow::Result<()> {
    let config = Config::load_from_root(root_dir)?;
    
    // driver_patternsでマッチしたファイルを取得
    let driver_files = find_driver_matched_files(&config, root_dir)?;
    
    // command.testを優先し、なければrun_test（後方互換性）を使用
    let run_test = config.command
        .as_ref()
        .and_then(|c| c.test.as_ref())
        .or_else(|| config.run_test.as_ref())
        .ok_or_else(|| anyhow::anyhow!("[command.test] or [run_test] section not found in overcode.toml"))?;
    
    if driver_files.is_empty() {
        warn!("No files matched driver_patterns pattern. Nothing to test.");
        return Ok(());
    }
    
    info!("Found {} driver file(s) to test", driver_files.len());
    
    // 各ファイルに対して一つずつ実行
    let mut success_count = 0;
    let mut failure_count = 0;
    
    for driver_file in &driver_files {
        info!("Testing driver file: {}", driver_file);
        
        match execute_test_command(
            &run_test,
            driver_file,
            root_dir,
        ) {
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
#[path = "test/driver/config.rs"]
mod driver_config;

#[cfg(test)]
#[path = "test/driver/podman_mount.rs"]
mod driver_podman_mount;

