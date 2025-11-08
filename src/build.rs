use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml::Value;
use crate::storage::Storage;

/// ビルド操作を実行する
pub fn process_build(
    storage: &Storage,
) -> Result<()> {
    // 最新のhistoryファイルの内容とタイムスタンプを取得
    let (timestamp, history_content) = storage.load_latest_history_content()
        .context("Failed to load latest history file")?;
    
    println!("Building from history file with timestamp: {}", timestamp);
    
    // .overcode/builds/{timestamp}/ディレクトリを作成
    let builds_dir = storage.overcode_dir().join("builds").join(timestamp.to_string());
    fs::create_dir_all(&builds_dir)
        .context("Failed to create builds directory")?;
    
    // ファイルを展開（シンボリックリンクで）
    expand_files_from_history(&history_content, storage, &builds_dir)
        .context("Failed to expand files from history")?;
    
    println!("Files expanded to: {:?}", builds_dir);
    
    // Cargo.lockファイルの処理
    // 既存のCargo.lockが新しいバージョンのクレートを含んでいる可能性があるため、
    // Cargo.tomlから新しいCargo.lockを生成する（BazelのCargoバージョンと互換性のあるバージョンが選択される）
    let cargo_toml_path = builds_dir.join("Cargo.toml");
    let target_cargo_lock = builds_dir.join("Cargo.lock");
    
    if cargo_toml_path.exists() {
        // 既存のCargo.lockを削除（新しいバージョンのクレートが含まれている可能性があるため）
        if target_cargo_lock.exists() {
            fs::remove_file(&target_cargo_lock)
                .context("Failed to remove existing Cargo.lock")?;
        }
        
        // Cargo.tomlから新しいCargo.lockを生成
        // これにより、Bazelが使用するCargoのバージョンと互換性のあるバージョンが選択される
        let generate_output = Command::new("cargo")
            .arg("generate-lockfile")
            .arg("--manifest-path")
            .arg(&cargo_toml_path)
            .current_dir(&builds_dir)
            .output()
            .context("Failed to execute cargo generate-lockfile")?;
        
        if !generate_output.status.success() {
            // cargo generate-lockfileが失敗した場合、警告を出して続行
            eprintln!("Warning: Failed to generate Cargo.lock: {}", String::from_utf8_lossy(&generate_output.stderr));
            eprintln!("crate_universe will attempt to resolve dependencies automatically");
        } else {
            println!("Cargo.lock generated from Cargo.toml for Bazel compatibility");
            
            // Cargo.lockファイルを読み込んで、globsetのバージョンを0.4.17以下に制限
            // globset-0.4.18はedition2024を要求するため、BazelのCargoバージョンと互換性がない
            if target_cargo_lock.exists() {
                let lock_content = fs::read_to_string(&target_cargo_lock)
                    .context("Failed to read Cargo.lock")?;
                
                // globset-0.4.18を0.4.17に置き換え
                let fixed_content = lock_content
                    .replace("version = \"0.4.18\"", "version = \"0.4.17\"")
                    .replace("globset 0.4.18", "globset 0.4.17");
                
                if fixed_content != lock_content {
                    fs::write(&target_cargo_lock, fixed_content)
                        .context("Failed to write fixed Cargo.lock")?;
                    println!("Fixed globset version in Cargo.lock to 0.4.17 for Bazel compatibility");
                    println!("Note: This may cause dependency resolution issues, but is necessary for Bazel compatibility");
                }
            }
        }
    }
    
    // BUILDファイルを生成
    let build_content = generate_build_file(&history_content, storage)
        .context("Failed to generate BUILD file")?;
    
    // builds/{timestamp}/BUILDに保存
    let build_path = builds_dir.join("BUILD");
    fs::write(&build_path, build_content)
        .context("Failed to write BUILD file")?;
    
    println!("BUILD file written to: {:?}", build_path);
    
    // MODULE.bazelファイルを生成（Bazelワークスペースに必要）
    let module_bazel_path = builds_dir.join("MODULE.bazel");
    
    // Cargo.tomlが存在するか確認して、crate_universeの設定を追加
    let cargo_toml_path = builds_dir.join("Cargo.toml");
    let has_cargo_toml = cargo_toml_path.exists();
    
    let module_bazel_content = if has_cargo_toml {
        // Cargo.tomlがある場合、crate_universeを使って依存関係を解決
        // 空のCargo.lockファイルを指定して、crate_universeに自動解決させる
        // これにより、Bazelが使用するCargoのバージョンと互換性のあるバージョンが選択される
        r#"module(name = "overcode")

bazel_dep(name = "rules_rust", version = "0.67.0")

crate = use_extension("@rules_rust//crate_universe:extension.bzl", "crate")

crate.from_cargo(
    name = "crates",
    cargo_lockfile = "//:Cargo.lock",
    manifests = ["//:Cargo.toml"],
)

use_repo(crate, "crates")
"#
    } else {
        // Cargo.tomlがない場合、基本的な設定のみ
        r#"module(name = "overcode")

bazel_dep(name = "rules_rust", version = "0.67.0")
"#
    };
    
    fs::write(&module_bazel_path, module_bazel_content)
        .context("Failed to write MODULE.bazel file")?;
    
    println!("MODULE.bazel file written to: {:?}", module_bazel_path);
    println!("Build operation completed!");
    
    Ok(())
}

/// historyファイルから各ファイルのハッシュを取得し、builds/{timestamp}/ディレクトリ内に実際のファイル構造を再現
/// シンボリックリンクを使用して.overcode/{hash}からbuilds/{timestamp}/{file_path}へのリンクを作成
fn expand_files_from_history(
    history_content: &str,
    storage: &Storage,
    builds_dir: &Path,
) -> Result<()> {
    // historyファイルを解析
    let history_value: Value = toml::from_str(history_content)
        .context("Failed to parse history file")?;
    
    // ファイル一覧を取得
    let files = history_value.get("files")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow::anyhow!("Invalid history file format: missing 'files' section"))?;
    
    // .overcodeディレクトリのパスを取得
    let overcode_dir = storage.overcode_dir();
    
    // 各ファイルを展開
    for (file_path, file_data) in files {
        if let Some(table) = file_data.as_table() {
            if let Some(hash) = table.get("hash").and_then(|v| v.as_str()) {
                if !hash.is_empty() {
                    // ターゲットファイルのパスを作成
                    let target_path = builds_dir.join(&file_path);
                    
                    // ディレクトリを作成
                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)
                            .with_context(|| format!("Failed to create directory: {:?}", parent))?;
                    }
                    
                    // 既存のファイルやシンボリックリンクを削除（存在する場合）
                    if target_path.exists() || target_path.is_symlink() {
                        fs::remove_file(&target_path)
                            .with_context(|| format!("Failed to remove existing file: {:?}", target_path))?;
                    }
                    
                    // ソースファイルのパス（.overcode/{hash}）
                    let source_path = overcode_dir.join(hash);
                    
                    // ソースファイルが存在するか確認
                    if !source_path.exists() {
                        return Err(anyhow::anyhow!("Source file not found: {:?} (hash: {})", source_path, hash));
                    }
                    
                    // ターゲットからソースへの相対パスを計算
                    let target_parent = target_path.parent()
                        .ok_or_else(|| anyhow::anyhow!("Target path has no parent: {:?}", target_path))?;
                    let source_relative = calculate_relative_path(target_parent, &source_path)
                        .with_context(|| format!("Failed to calculate relative path from {:?} to {:?}", target_parent, source_path))?;
                    
                    // シンボリックリンクを作成
                    #[cfg(unix)]
                    {
                        std::os::unix::fs::symlink(&source_relative, &target_path)
                            .with_context(|| format!("Failed to create symlink from {:?} to {:?}", target_path, source_relative))?;
                    }
                    #[cfg(windows)]
                    {
                        // Windowsでは、ディレクトリかファイルかによって異なる関数を使う必要がある
                        if source_path.is_dir() {
                            std::os::windows::fs::symlink_dir(&source_relative, &target_path)
                                .with_context(|| format!("Failed to create symlink from {:?} to {:?}", target_path, source_relative))?;
                        } else {
                            std::os::windows::fs::symlink_file(&source_relative, &target_path)
                                .with_context(|| format!("Failed to create symlink from {:?} to {:?}", target_path, source_relative))?;
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// historyファイルの内容からBUILDファイルの内容を生成する（副作用なし）
fn generate_build_file(history_content: &str, storage: &Storage) -> Result<String> {
    // historyファイルを解析
    let history_value: Value = toml::from_str(history_content)
        .context("Failed to parse history file")?;
    
    // ファイル一覧を取得
    let files = history_value.get("files")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow::anyhow!("Invalid history file format: missing 'files' section"))?;
    
    // Cargo.tomlのハッシュを取得
    let cargo_toml_hash = files.get("Cargo.toml")
        .and_then(|v| v.as_table())
        .and_then(|t| t.get("hash"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml not found in history file"))?;
    
    // Cargo.tomlの内容を読み込む
    let cargo_toml_content = storage.load_file(cargo_toml_hash)
        .context("Failed to load Cargo.toml")?;
    let cargo_toml_str = String::from_utf8(cargo_toml_content)
        .context("Failed to decode Cargo.toml as UTF-8")?;
    
    // Cargo.tomlを解析して依存関係を取得
    let cargo_toml_value: Value = toml::from_str(&cargo_toml_str)
        .context("Failed to parse Cargo.toml")?;
    
    let external_deps = extract_external_dependencies(&cargo_toml_value)?;
    
    // .rsファイルを収集
    let mut rust_files = Vec::new();
    let mut file_hashes = HashMap::new();
    
    for (path, file_data) in files {
        if path.ends_with(".rs") || path == "Cargo.toml" || path == "build.rs" {
            if let Some(table) = file_data.as_table() {
                if let Some(hash) = table.get("hash").and_then(|v| v.as_str()) {
                    if !hash.is_empty() {
                        rust_files.push(path.clone());
                        file_hashes.insert(path.clone(), hash.to_string());
                    }
                }
            }
        }
    }
    
    // main.rsが存在するか確認
    if !rust_files.iter().any(|p| p == "src/main.rs") {
        return Err(anyhow::anyhow!("src/main.rs not found"));
    }
    
    // 各ファイルの依存関係を構築（メタ情報から取得）
    let mut file_deps: HashMap<String, Vec<String>> = HashMap::new();
    for (path, hash) in &file_hashes {
        if path.ends_with(".rs") {
            let entries = storage.load_meta(hash).unwrap_or_default();
            let mut deps = Vec::new();
            for entry in entries {
                for (dep_path, _) in entry.deps {
                    if dep_path.ends_with(".rs") {
                        deps.push(dep_path);
                    }
                }
            }
            deps.sort();
            deps.dedup();
            file_deps.insert(path.clone(), deps);
        }
    }
    
    // BUILDファイルを生成
    generate_bazel_build_file(&rust_files, &file_deps, &external_deps)
}

/// Cargo.tomlから外部依存関係を抽出
fn extract_external_dependencies(cargo_toml: &Value) -> Result<Vec<String>> {
    let mut deps = Vec::new();
    
    if let Some(dependencies) = cargo_toml.get("dependencies").and_then(|v| v.as_table()) {
        for (name, _) in dependencies {
            deps.push(name.clone());
        }
    }
    
    deps.sort();
    Ok(deps)
}

/// BazelのBUILDファイルを生成
fn generate_bazel_build_file(
    rust_files: &[String],
    file_deps: &HashMap<String, Vec<String>>,
    external_deps: &[String],
) -> Result<String> {
    let mut build = String::new();
    
    // インポート
    build.push_str("load(\"@rules_rust//rust:defs.bzl\", \"rust_binary\", \"rust_test\")\n\n");
    
    // 外部依存関係のリスト
    let external_deps_str: Vec<String> = external_deps.iter()
        .map(|dep| format!("        \"@crates//:{}\"", dep))
        .collect();
    
    // rust_binaryターゲット
    let srcs: Vec<String> = rust_files.iter()
        .filter(|p| p.ends_with(".rs"))
        .map(|p| format!("        \"{}\"", p))
        .collect();
    
    build.push_str("rust_binary(\n");
    build.push_str("    name = \"overcode\",\n");
    build.push_str("    srcs = [\n");
    build.push_str(&srcs.join(",\n"));
    build.push_str("\n    ],\n");
    if !external_deps_str.is_empty() {
        build.push_str("    deps = [\n");
        build.push_str(&external_deps_str.join(",\n"));
        build.push_str("\n    ],\n");
    }
    build.push_str("    edition = \"2021\",\n");
    build.push_str(")\n\n");
    
    // 各.rsファイルに対してrust_testターゲットを生成
    for file in rust_files {
        if file.ends_with(".rs") && file != "src/main.rs" {
            let test_name = file
                .trim_start_matches("src/")
                .trim_end_matches(".rs")
                .replace("/", "_")
                .replace("-", "_");
            
            // テストファイルの依存関係を取得
            let mut test_deps = file_deps.get(file).cloned().unwrap_or_default();
            test_deps.push(file.clone());
            test_deps.sort();
            test_deps.dedup();
            
            let test_srcs: Vec<String> = test_deps.iter()
                .map(|p| format!("        \"{}\"", p))
                .collect();
            
            let test_external_deps: Vec<String> = external_deps.iter()
                .map(|dep| format!("        \"@crates//:{}\"", dep))
                .collect();
            
            build.push_str(&format!("rust_test(\n"));
            build.push_str(&format!("    name = \"{}_test\",\n", test_name));
            build.push_str("    srcs = [\n");
            build.push_str(&test_srcs.join(",\n"));
            build.push_str("\n    ],\n");
            if !test_external_deps.is_empty() {
                build.push_str("    deps = [\n");
                build.push_str(&test_external_deps.join(",\n"));
                build.push_str("\n    ],\n");
            }
            build.push_str("    edition = \"2021\",\n");
            build.push_str(")\n\n");
        }
    }
    
    Ok(build)
}

/// 2つのパス間の相対パスを計算する
fn calculate_relative_path(from: &Path, to: &Path) -> Result<PathBuf> {
    let from = from.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", from))?;
    let to = to.canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {:?}", to))?;
    
    let mut from_components: Vec<_> = from.components().collect();
    let mut to_components: Vec<_> = to.components().collect();
    
    // 共通のプレフィックスを削除
    while !from_components.is_empty() && !to_components.is_empty() {
        if from_components[0] == to_components[0] {
            from_components.remove(0);
            to_components.remove(0);
        } else {
            break;
        }
    }
    
    // fromから共通の親ディレクトリまでの相対パスを構築
    let mut relative = PathBuf::new();
    for _ in from_components {
        relative.push("..");
    }
    
    // toの残りのコンポーネントを追加
    for component in to_components {
        relative.push(component);
    }
    
    Ok(relative)
}

/// テスト操作を実行する
pub fn process_test(
    storage: &Storage,
) -> Result<()> {
    // 最新のhistoryファイルの内容とタイムスタンプを取得
    let (timestamp, _) = storage.load_latest_history_content()
        .context("Failed to load latest history file")?;
    
    println!("Testing from history file with timestamp: {}", timestamp);
    
    // .overcode/builds/{timestamp}/ディレクトリとBUILDファイルの存在を確認
    let builds_dir = storage.overcode_dir().join("builds").join(timestamp.to_string());
    let build_path = builds_dir.join("BUILD");
    
    if !builds_dir.exists() {
        return Err(anyhow::anyhow!("Build directory not found: {:?}. Please run 'build' command first.", builds_dir));
    }
    
    if !build_path.exists() {
        return Err(anyhow::anyhow!("BUILD file not found: {:?}. Please run 'build' command first.", build_path));
    }
    
    println!("Found build directory: {:?}", builds_dir);
    println!("Found BUILD file: {:?}", build_path);
    
    // MODULE.bazelが存在する場合、先にbazel buildを実行してモジュールを解決
    // これにより、モジュール拡張が正しく初期化される
    let module_bazel_path = builds_dir.join("MODULE.bazel");
    if module_bazel_path.exists() {
        println!("Found MODULE.bazel, running bazel build to resolve modules...");
        let build_output = Command::new("bazel")
            .arg("build")
            .arg("//...")
            .current_dir(&builds_dir)
            .output()
            .context("Failed to execute bazel build command")?;
        
        if !build_output.stdout.is_empty() {
            print!("{}", String::from_utf8_lossy(&build_output.stdout));
        }
        if !build_output.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&build_output.stderr));
        }
        
        // buildが失敗した場合はエラーを返す（モジュール解決に失敗している可能性がある）
        if !build_output.status.success() {
            return Err(anyhow::anyhow!("Bazel build command failed with exit code: {:?}. This may indicate module resolution issues.", build_output.status.code()));
        }
    }
    
    // builds/{timestamp}/ディレクトリでbazel test //...コマンドを実行
    println!("Running bazel test //... in {:?}", builds_dir);
    
    let output = Command::new("bazel")
        .arg("test")
        .arg("//...")
        .current_dir(&builds_dir)
        .output()
        .context("Failed to execute bazel test command")?;
    
    // 標準出力と標準エラー出力を表示
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("Bazel test command failed with exit code: {:?}", output.status.code()));
    }
    
    println!("Test operation completed!");
    
    Ok(())
}

