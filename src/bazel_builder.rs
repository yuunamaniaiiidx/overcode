use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;
use toml::Value;
use crate::storage::Storage;

/// Cargo.tomlから取得した情報
struct CargoInfo {
    edition: String,
    version: String,
    dependencies: Vec<String>,
}

/// Cargo.tomlを解析して情報を取得
fn parse_cargo_toml(root_dir: &Path) -> Result<CargoInfo> {
    let cargo_toml_path = root_dir.join("Cargo.toml");
    
    if !cargo_toml_path.exists() {
        // Cargo.tomlが存在しない場合はデフォルト値を返す
        return Ok(CargoInfo {
            edition: "2021".to_string(),
            version: "1.87.0".to_string(),
            dependencies: Vec::new(),
        });
    }
    
    let content = fs::read_to_string(&cargo_toml_path)
        .context("Failed to read Cargo.toml")?;
    let value: Value = toml::from_str(&content)
        .context("Failed to parse Cargo.toml")?;
    
    // editionを取得（デフォルト: "2021"）
    let edition = value
        .get("package")
        .and_then(|p| p.get("edition"))
        .and_then(|e| e.as_str())
        .unwrap_or("2021")
        .to_string();
    
    // RustバージョンはCargo.tomlからは取得できないので、システムのRustバージョンを取得
    let version = get_rust_version().unwrap_or_else(|| "1.87.0".to_string());
    
    // dependenciesを取得
    let mut dependencies = Vec::new();
    if let Some(deps) = value.get("dependencies").and_then(|d| d.as_table()) {
        for (name, _) in deps {
            dependencies.push(name.clone());
        }
    }
    
    Ok(CargoInfo {
        edition,
        version,
        dependencies,
    })
}

/// システムのRustバージョンを取得
fn get_rust_version() -> Option<String> {
    let output = ProcessCommand::new("rustc")
        .arg("--version")
        .output()
        .ok()?;
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    // "rustc 1.87.0 (17067e9ac 2025-05-09)" から "1.87.0" を抽出
    if let Some(start) = version_str.find(' ') {
        let rest = &version_str[start + 1..];
        if let Some(end) = rest.find(' ') {
            return Some(rest[..end].to_string());
        }
    }
    None
}

/// BAZEL BUILDファイルとWORKSPACEファイルを生成する
/// include_testsがtrueの場合、.rsファイルに対してrust_testターゲットを生成し、WORKSPACEにrules_rustを追加する
pub fn generate_build_files(
    root_dir: &Path,
    history_path: &Path,
    include_tests: bool,
) -> Result<(PathBuf, PathBuf)> {
    // Cargo.tomlを解析
    let cargo_info = parse_cargo_toml(root_dir)?;
    
    // historyファイルを読み込む
    let content = fs::read_to_string(history_path)?;
    let value: Value = toml::from_str(&content)?;

    // buildsディレクトリを作成
    let overcode_dir = root_dir.join(".overcode");
    let builds_dir = overcode_dir.join("builds");
    
    // 既存のbuildsディレクトリをクリーンアップ（シンボリックリンクを削除）
    if builds_dir.exists() {
        remove_dir_contents(&builds_dir)?;
    }
    fs::create_dir_all(&builds_dir)?;

    // ファイルリストを収集（パスとハッシュのペア）
    let mut file_entries: Vec<(String, String)> = Vec::new();
    if let Some(files) = value.get("files").and_then(|v| v.as_table()) {
        for (path, file_data) in files {
            if let Some(table) = file_data.as_table() {
                if let Some(hash) = table.get("hash").and_then(|v| v.as_str()) {
                    if !hash.is_empty() {
                        // ハッシュファイルが存在するか確認
                        let hash_file_path = overcode_dir.join(hash);
                        if hash_file_path.exists() {
                            file_entries.push((path.clone(), hash.to_string()));
                        }
                    }
                }
            }
        }
    }

    // シンボリックリンクを作成（パス構造を復元）
    for (path, hash) in &file_entries {
        let link_path = builds_dir.join(path);
        
        // ディレクトリを作成
        if let Some(parent) = link_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 既存のリンクやファイルを削除
        if link_path.exists() || link_path.is_symlink() {
            fs::remove_file(&link_path).ok();
        }
        
        // 相対パスでシンボリックリンクを作成
        // builds/src/main.rs から ../{hash} へのリンク
        // builds/Cargo.lock から ../{hash} へのリンク
        let link_depth = path.matches('/').count();
        let relative_hash_path = if link_depth == 0 {
            // ルートファイル（例：Cargo.lock）の場合、../{hash}
            format!("../{}", hash)
        } else {
            // サブディレクトリのファイル（例：src/main.rs）の場合、../../{hash}
            format!("{}{}", "../".repeat(link_depth + 1), hash)
        };
        symlink(&relative_hash_path, &link_path)?;
    }

    // BUILDファイルのパス
    let build_file_path = builds_dir.join("BUILD");

    // BUILDファイルの内容を生成
    let mut build_content = String::new();
    build_content.push_str("# Generated BUILD file from history\n");
    build_content.push_str("# DO NOT EDIT MANUALLY\n\n");
    
    // テストターゲットを生成する場合のみloadを追加
    if include_tests {
        build_content.push_str("load(\"@rules_rust//rust:defs.bzl\", \"rust_test\")\n\n");
    }

    // filegroupルールを生成（シンボリックリンクされたファイルを参照）
    if !file_entries.is_empty() {
        build_content.push_str("filegroup(\n");
        build_content.push_str("    name = \"sources\",\n");
        build_content.push_str("    srcs = [\n");
        for (path, _) in &file_entries {
            // パスをBAZEL形式に変換（パス区切り文字を統一）
            let normalized_path = path.replace('\\', "/");
            // buildsディレクトリがワークスペースルートなので、相対パスで参照
            build_content.push_str(&format!("        \"{}\",\n", normalized_path));
        }
        build_content.push_str("    ],\n");
        build_content.push_str("    visibility = [\"//visibility:public\"],\n");
        build_content.push_str(")\n\n");
    }

    // .rsファイルに対してrust_testターゲットを生成（include_testsがtrueの場合のみ）
    if include_tests {
        // crates_vendorルールを追加（依存関係を自動解決）
        if !cargo_info.dependencies.is_empty() {
            build_content.push_str("load(\"@rules_rust//crate_universe:defs.bzl\", \"crates_vendor\")\n\n");
            build_content.push_str("crates_vendor(\n");
            build_content.push_str("    name = \"crates\",\n");
            build_content.push_str("    cargo_lockfile = \"//:Cargo.lock\",\n");
            build_content.push_str("    manifests = [\"//:Cargo.toml\"],\n");
            build_content.push_str("    mode = \"local\",\n");
            build_content.push_str("    tags = [\"manual\"],\n");
            build_content.push_str(")\n\n");
        }
        
        let mut rust_files: Vec<&String> = file_entries
            .iter()
            .map(|(path, _)| path)
            .filter(|path| path.ends_with(".rs"))
            .collect();
        rust_files.sort();

        // 依存関係を文字列に変換
        let deps_str = if cargo_info.dependencies.is_empty() {
            String::new()
        } else {
            let deps_list: Vec<String> = cargo_info.dependencies
                .iter()
                .map(|dep| format!("        \"@crates//:{}\"", dep))
                .collect();
            format!("    deps = [\n{}\n    ],\n", deps_list.join(",\n"))
        };

        for path in &rust_files {
            let normalized_path = path.replace('\\', "/");
            // ターゲット名を生成（パスから拡張子を除き、スラッシュをアンダースコアに変換）
            let target_name = normalized_path
                .trim_end_matches(".rs")
                .replace('/', "_")
                .replace('-', "_")
                .replace('.', "_");
            
            build_content.push_str(&format!("rust_test(\n"));
            build_content.push_str(&format!("    name = \"{}_test\",\n", target_name));
            build_content.push_str(&format!("    srcs = [\"{}\"],\n", normalized_path));
            if !deps_str.is_empty() {
                build_content.push_str(&deps_str);
            }
            build_content.push_str(&format!("    visibility = [\"//visibility:public\"],\n"));
            build_content.push_str(&format!(")\n\n"));
        }
    }

    // BUILDファイルを書き込む
    fs::write(&build_file_path, build_content)?;

    // WORKSPACEファイルを生成（builds/WORKSPACEに）
    let workspace_path = builds_dir.join("WORKSPACE");
    let mut workspace_content = String::new();
    workspace_content.push_str("# Generated WORKSPACE file\n");
    workspace_content.push_str("# DO NOT EDIT MANUALLY\n\n");
    workspace_content.push_str("workspace(name = \"overcode\")\n");
    fs::write(&workspace_path, workspace_content)?;

    // テストターゲットを生成する場合のみMODULE.bazelを生成（Bazel 8対応）
    let _module_path = if include_tests {
        let module_file_path = builds_dir.join("MODULE.bazel");
        let mut module_content = String::new();
        module_content.push_str("# Generated MODULE.bazel file\n");
        module_content.push_str("# DO NOT EDIT MANUALLY\n\n");
        module_content.push_str("module(\n");
        module_content.push_str("    name = \"overcode\",\n");
        module_content.push_str("    version = \"0.0.0\",\n");
        module_content.push_str(")\n\n");
        module_content.push_str("bazel_dep(name = \"rules_rust\", version = \"0.40.0\")\n\n");
        
        // Rustツールチェーンの設定
        module_content.push_str("rust = use_extension(\"@rules_rust//rust:extensions.bzl\", \"rust\")\n");
        module_content.push_str(&format!("rust.toolchain(\n"));
        module_content.push_str(&format!("    edition = \"{}\",\n", cargo_info.edition));
        module_content.push_str(&format!("    versions = [\"{}\"],\n", cargo_info.version));
        module_content.push_str(")\n");
        module_content.push_str("use_repo(rust, \"rust_toolchains\")\n");
        module_content.push_str("register_toolchains(\"@rust_toolchains//:all\")\n");
        
        fs::write(&module_file_path, module_content)?;
        Some(module_file_path)
    } else {
        None
    };

    Ok((build_file_path, workspace_path))
}

/// ディレクトリの内容を再帰的に削除（シンボリックリンクも含む）
fn remove_dir_contents(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_symlink() || path.is_file() {
            fs::remove_file(&path)?;
        } else if path.is_dir() {
            fs::remove_dir_all(&path)?;
        }
    }
    
    Ok(())
}

/// Buildコマンドの処理を実行する
/// 最新のhistoryファイルを取得し、BUILDファイルとWORKSPACEファイルを生成してBAZELビルドを実行する
pub fn process_build(root_dir: &Path) -> Result<()> {
    // 最新のhistoryファイルを取得
    let storage = Storage::new(root_dir)?;
    let latest_history = storage.get_latest_history_path()?;
    
    match latest_history {
        Some((_timestamp, history_path)) => {
            // BUILDファイルとWORKSPACEファイルを生成（テストターゲットは含めない）
            let (build_file_path, workspace_path) = generate_build_files(
                root_dir,
                &history_path,
                false,
            )?;
            
            println!("Generated BUILD file at: {:?}", build_file_path);
            println!("Generated WORKSPACE file at: {:?}", workspace_path);
            
            // BAZELコマンドを実行（.overcode/buildsディレクトリから）
            let builds_dir = root_dir.join(".overcode").join("builds");
            
            let output = ProcessCommand::new("bazel")
                .arg("build")
                .arg("//:sources")
                .current_dir(&builds_dir)
                .output()
                .context("Failed to execute bazel command")?;
            
            // 標準出力と標準エラー出力を表示
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
            
            if !output.status.success() {
                anyhow::bail!("BAZEL build failed with exit code: {:?}", output.status.code());
            }
            
            println!("BAZEL build completed successfully");
            Ok(())
        }
        None => {
            anyhow::bail!("No history file found. Please run 'index' command first.");
        }
    }
}

/// Testコマンドの処理を実行する
/// 最新のhistoryファイルを取得し、BUILDファイルとWORKSPACEファイルを生成してBAZELテストを実行する
pub fn process_test_target(root_dir: &Path) -> Result<()> {
    // 最新のhistoryファイルを取得
    let storage = Storage::new(root_dir)?;
    let latest_history = storage.get_latest_history_path()?;
    
    match latest_history {
        Some((_timestamp, history_path)) => {
            // BUILDファイルとWORKSPACEファイルを生成（テストターゲットを含める）
            let (build_file_path, workspace_path) = generate_build_files(
                root_dir,
                &history_path,
                true,
            )?;
            
            println!("Generated BUILD file at: {:?}", build_file_path);
            println!("Generated WORKSPACE file at: {:?}", workspace_path);
            
            // BAZELコマンドを実行（.overcode/buildsディレクトリから）
            let builds_dir = root_dir.join(".overcode").join("builds");
            
            // 依存関係がある場合、crates_vendorを実行
            let cargo_info = parse_cargo_toml(root_dir)?;
            if !cargo_info.dependencies.is_empty() {
                println!("Generating crate dependencies...");
                let vendor_output = ProcessCommand::new("bazel")
                    .arg("run")
                    .arg("//:crates")
                    .current_dir(&builds_dir)
                    .output()
                    .context("Failed to execute bazel run //:crates")?;
                
                if !vendor_output.stdout.is_empty() {
                    println!("{}", String::from_utf8_lossy(&vendor_output.stdout));
                }
                if !vendor_output.stderr.is_empty() {
                    eprintln!("{}", String::from_utf8_lossy(&vendor_output.stderr));
                }
                
                if !vendor_output.status.success() {
                    anyhow::bail!("Failed to generate crate dependencies");
                }
            }
            
            // .rsファイルのテストターゲットをすべて実行
            let output = ProcessCommand::new("bazel")
                .arg("test")
                .arg("//...")
                .current_dir(&builds_dir)
                .output()
                .context("Failed to execute bazel test command")?;
            
            // 標準出力と標準エラー出力を表示
            if !output.stdout.is_empty() {
                println!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
            
            if !output.status.success() {
                anyhow::bail!("BAZEL test failed with exit code: {:?}", output.status.code());
            }
            
            println!("BAZEL test completed successfully");
            Ok(())
        }
        None => {
            anyhow::bail!("No history file found. Please run 'index' command first.");
        }
    }
}

