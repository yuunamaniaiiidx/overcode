use anyhow::Context;
use std::path::Path;
use std::process::Command;
use std::io::Write;
use regex::Regex;
use crate::config::Config;
use crate::storage::Storage;
use log::{info, warn};

/// driver_patternsのパターンにマッチしたファイルを取得する
/// 元のファイル名（変換前）を返すことで、{src_name}.driver.{testtitle}.rs形式のファイルを個別に実行できる
fn find_driver_matched_files(config: &Config, storage: &Storage) -> anyhow::Result<Vec<String>> {
    let file_index = storage.load_index()
        .context("Failed to load index")?;
    
    let mut matched_files = Vec::new();
    
    // 各driver_patternパターンを適用
    for mapping in &config.driver_patterns {
        let pattern = Regex::new(&mapping.pattern)
            .with_context(|| format!("Invalid regex pattern: {}", mapping.pattern))?;
        
        for (file_path, _) in file_index.iter() {
            if pattern.is_match(file_path) {
                // 元のファイル名（変換前）を保持して、個別に実行できるようにする
                matched_files.push(file_path.clone());
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
    command: &str,
    args: &[String],
    driver_file: &str,
    root_dir: &Path,
) -> anyhow::Result<()> {
    // .overcode/buildsディレクトリのパスを取得
    let builds_dir = root_dir.join(".overcode").join("builds");
    let builds_dir_str = builds_dir.display().to_string();
    let root_dir_str = root_dir.display().to_string();
    
    // args内の{driver_file}、{root_dir}、{builds_dir}を置換
    let processed_args: Vec<String> = args
        .iter()
        .map(|arg| {
            arg.replace("{driver_file}", driver_file)
               .replace("{root_dir}", &root_dir_str)
               .replace("{builds_dir}", &builds_dir_str)
        })
        .collect();
    
    info!("Executing: {} {:?} (from {:?})", command, processed_args, builds_dir);
    
    // .overcode/buildsディレクトリからテストを実行
    let output = Command::new(command)
        .args(&processed_args)
        .current_dir(&builds_dir)
        .output()
        .with_context(|| format!("Failed to execute command: {}", command))?;
    
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
    
    let storage = Storage::new(root_dir)?;
    
    // driver_patternsでマッチしたファイルを取得
    let driver_files = find_driver_matched_files(&config, &storage)?;
    
    let run_test = config.run_test
        .ok_or_else(|| anyhow::anyhow!("[run_test] section not found in overcode.toml"))?;
    
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
            &run_test.command,
            &run_test.args,
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

