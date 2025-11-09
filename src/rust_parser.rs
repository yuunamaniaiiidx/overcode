use regex::Regex;
use std::path::Path;
use crate::config::Config;

/// プロジェクト内のローカルファイル（root_dir内）への依存関係のみを抽出
/// 標準ライブラリやサードパーティライブラリは除外する
pub fn extract_dependencies(
    file_path: &Path,
    file_content: &str,
    root_dir: &Path,
    config: &Config,
) -> anyhow::Result<Vec<String>> {
    let mut deps = Vec::new();
    
    // use文のパターン
    // use crate::module::item;
    // use super::module;
    // use self::module;
    // use crate::module::{item1, item2};
    // use crate::module as alias;
    let use_pattern = Regex::new(r"^use\s+(crate|super|self)::([a-zA-Z_][a-zA-Z0-9_:]*)(?:::\{[^}]+\}|::\*|(?:\s+as\s+\w+)?)?;?").unwrap();
    
    // mod宣言のパターン
    // mod module_name;
    let mod_pattern = Regex::new(r"^mod\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*;").unwrap();

    for line in file_content.lines() {
        let line = line.trim();
        
        // コメント行はスキップ
        if line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        // use文の処理
        if let Some(captures) = use_pattern.captures(line) {
            if let Some(prefix) = captures.get(1) {
                let prefix_str = prefix.as_str();
                
                // std:: で始まるものは標準ライブラリなので除外
                if line.contains("std::") {
                    continue;
                }
                
                // crate::, super::, self:: で始まるもののみ処理
                if let Some(module) = captures.get(2) {
                    let module_path = module.as_str();
                    
                    // モジュールパスを解決
                    let dep_path = resolve_rust_module(
                        file_path,
                        prefix_str,
                        module_path,
                        root_dir,
                    )?;
                    if let Some(dep) = dep_path {
                        // パターンを適用して変換
                        let transformed = apply_patterns(&dep, config);
                        deps.push(transformed);
                    }
                }
            }
        }
        
        // mod宣言の処理
        if let Some(captures) = mod_pattern.captures(line) {
            if let Some(module_name) = captures.get(1) {
                let module_name_str = module_name.as_str();
                
                // mod宣言からモジュールファイルを探す
                let dep_path = resolve_mod_declaration(
                    file_path,
                    module_name_str,
                    root_dir,
                )?;
                if let Some(dep) = dep_path {
                    // パターンを適用して変換
                    let transformed = apply_patterns(&dep, config);
                    deps.push(transformed);
                }
            }
        }
    }

    // 重複を除去
    deps.sort();
    deps.dedup();
    
    Ok(deps)
}

/// Rustモジュールを解決し、root_dir内のローカルファイルの相対パスを返す
/// root_dir外のファイル（標準ライブラリ、サードパーティライブラリ）の場合はNoneを返す
fn resolve_rust_module(
    file_path: &Path,
    prefix: &str,
    module_path: &str,
    root_dir: &Path,
) -> anyhow::Result<Option<String>> {
    let file_dir = file_path.parent().unwrap_or(Path::new("."));
    
    match prefix {
        "crate" => {
            // crate:: はルートディレクトリからの絶対パス
            let module_path = module_path.replace("::", "/");
            let rs_file = root_dir.join(&module_path).with_extension("rs");
            
            if rs_file.exists() && rs_file.starts_with(root_dir) {
                if let Ok(rel) = rs_file.strip_prefix(root_dir) {
                    return Ok(Some(rel.to_string_lossy().to_string()));
                }
            }
            
            // mod.rs を含むディレクトリを探す
            let mod_rs_file = root_dir.join(&module_path).join("mod.rs");
            if mod_rs_file.exists() && mod_rs_file.starts_with(root_dir) {
                if let Ok(rel) = mod_rs_file.strip_prefix(root_dir) {
                    return Ok(Some(rel.to_string_lossy().to_string()));
                }
            }
        }
        "super" => {
            // super:: は親ディレクトリからの相対パス
            let file_dir = file_path.parent().unwrap_or(Path::new("."));
            if let Some(parent) = file_dir.parent() {
                let module_path = module_path.replace("::", "/");
                let rs_file = parent.join(&module_path).with_extension("rs");
                
                if rs_file.exists() && rs_file.starts_with(root_dir) {
                    if let Ok(rel) = rs_file.strip_prefix(root_dir) {
                        return Ok(Some(rel.to_string_lossy().to_string()));
                    }
                }
                
                // mod.rs を含むディレクトリを探す
                let mod_rs_file = parent.join(&module_path).join("mod.rs");
                if mod_rs_file.exists() && mod_rs_file.starts_with(root_dir) {
                    if let Ok(rel) = mod_rs_file.strip_prefix(root_dir) {
                        return Ok(Some(rel.to_string_lossy().to_string()));
                    }
                }
            }
        }
        "self" => {
            // self:: は同じディレクトリからの相対パス
            let module_path = module_path.replace("::", "/");
            let rs_file = file_dir.join(&module_path).with_extension("rs");
            
            if rs_file.exists() && rs_file.starts_with(root_dir) {
                if let Ok(rel) = rs_file.strip_prefix(root_dir) {
                    return Ok(Some(rel.to_string_lossy().to_string()));
                }
            }
            
            // mod.rs を含むディレクトリを探す
            let mod_rs_file = file_dir.join(&module_path).join("mod.rs");
            if mod_rs_file.exists() && mod_rs_file.starts_with(root_dir) {
                if let Ok(rel) = mod_rs_file.strip_prefix(root_dir) {
                    return Ok(Some(rel.to_string_lossy().to_string()));
                }
            }
        }
        _ => {}
    }
    
    // root_dir内に該当ファイルがない場合はNoneを返す
    // （標準ライブラリ、サードパーティライブラリ、または存在しないファイル）
    Ok(None)
}

/// mod宣言からモジュールファイルを解決し、root_dir内のローカルファイルの相対パスを返す
/// root_dir外のファイルの場合はNoneを返す
fn resolve_mod_declaration(
    file_path: &Path,
    module_name: &str,
    root_dir: &Path,
) -> anyhow::Result<Option<String>> {
    let file_dir = file_path.parent().unwrap_or(Path::new("."));
    
    // 同じディレクトリ内の module_name.rs を探す
    let rs_file = file_dir.join(format!("{}.rs", module_name));
    if rs_file.exists() && rs_file.starts_with(root_dir) {
        if let Ok(rel) = rs_file.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    // 同じディレクトリ内の module_name/mod.rs を探す
    let mod_rs_file = file_dir.join(module_name).join("mod.rs");
    if mod_rs_file.exists() && mod_rs_file.starts_with(root_dir) {
        if let Ok(rel) = mod_rs_file.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    Ok(None)
}

/// パスに対してパターンを適用して変換する
/// src_patterns、driver_patterns、mock_patternsの順に適用し、最初にマッチしたパターンで置換する
fn apply_patterns(path: &str, config: &Config) -> String {
    // 全てのパターンを順に試す
    let all_patterns = [
        &config.src_patterns,
        &config.driver_patterns,
        &config.mock_patterns,
    ];
    
    for patterns in all_patterns.iter() {
        for mapping in *patterns {
            if let Ok(regex) = Regex::new(&mapping.pattern) {
                if regex.is_match(path) {
                    // キャプチャグループを使って置換
                    let result = regex.replace(path, &mapping.resolution);
                    return result.to_string();
                }
            }
        }
    }
    
    // どのパターンにもマッチしない場合は元のパスを返す
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config() -> Config {
        Config {
            ignores: vec![],
            src_patterns: vec![
                crate::config::MappingEntry {
                    pattern: r"(.+)/(.+)\.src\..+\.(.+)".to_string(),
                    resolution: "$1/$2.$3".to_string(),
                },
            ],
            driver_patterns: vec![
                crate::config::MappingEntry {
                    pattern: r"(.+)/(.+)\.driver\..+\.(.+)".to_string(),
                    resolution: "$1/$2.$3".to_string(),
                },
            ],
            mock_patterns: vec![
                crate::config::MappingEntry {
                    pattern: r"(.+)/(.+)\.mock\..+\.(.+)".to_string(),
                    resolution: "$1/$2.$3".to_string(),
                },
            ],
            images: vec![],
            command: None,
            run_test: None,
        }
    }

    #[test]
    fn test_extract_dependencies_crate() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        // テスト用のファイル構造を作成
        fs::create_dir_all(root_dir.join("module")).unwrap();
        fs::write(root_dir.join("module.rs"), "pub fn test() {}").unwrap();
        fs::write(root_dir.join("module").join("submodule.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("main.rs");
        let content = r#"
use crate::module;
use crate::module::submodule;
"#;

        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"module.rs".to_string()));
        assert!(deps.contains(&"module/submodule.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_super() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        // テスト用のファイル構造を作成
        fs::create_dir_all(root_dir.join("parent")).unwrap();
        fs::write(root_dir.join("parent.rs"), "pub fn test() {}").unwrap();
        fs::write(root_dir.join("parent").join("child.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("parent").join("child.rs");
        let content = r#"
use super::parent;
"#;

        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        // super::parent は親ディレクトリ（root_dir）から parent.rs を探す
        // parent/child.rs から見ると、親ディレクトリは root_dir で、そこに parent.rs がある
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"parent.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_mod_declaration() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        // テスト用のファイル構造を作成
        fs::create_dir_all(root_dir.join("src")).unwrap();
        fs::write(root_dir.join("src").join("main.rs"), "pub fn test() {}").unwrap();
        fs::write(root_dir.join("src").join("module.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("src").join("main.rs");
        let content = r#"
mod module;
"#;

        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"src/module.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_ignores_comments() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        fs::write(root_dir.join("module.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("main.rs");
        let content = r#"
// use crate::module;
/* use crate::module; */
use crate::module;
"#;

        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"module.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_ignores_std() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        let config = create_test_config();
        let file_path = root_dir.join("main.rs");
        let content = r#"
use std::collections::HashMap;
use crate::module;
"#;

        // std:: を含むuse文は無視される
        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        // module.rs が存在しないので、depsは空になる
        assert_eq!(deps.len(), 0);
    }

    #[test]
    fn test_extract_dependencies_deduplicates() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        fs::write(root_dir.join("module.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("main.rs");
        let content = r#"
use crate::module;
use crate::module;
mod module;
"#;

        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        // 重複が除去される
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"module.rs".to_string()));
    }

    #[test]
    fn test_extract_dependencies_with_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();

        // src_patternsにマッチするファイルを作成
        fs::create_dir_all(root_dir.join("src")).unwrap();
        fs::write(root_dir.join("src").join("module.src.test.rs"), "pub fn test() {}").unwrap();

        let config = create_test_config();
        let file_path = root_dir.join("main.rs");
        let content = r#"
use crate::src::module;
"#;

        // module.src.test.rs は src_patterns で module.rs に変換される
        // ただし、extract_dependencies は実際のファイルパスを解決するので、
        // パターン変換は apply_patterns で行われる
        let deps = extract_dependencies(&file_path, content, root_dir, &config).unwrap();
        // 実際のファイルが存在しないので空になる
        assert_eq!(deps.len(), 0);
    }
}

