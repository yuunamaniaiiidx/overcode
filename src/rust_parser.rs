use regex::Regex;
use std::path::Path;

/// プロジェクト内のローカルファイル（root_dir内）への依存関係のみを抽出
/// 標準ライブラリやサードパーティライブラリは除外する
pub fn extract_dependencies(
    file_path: &Path,
    file_content: &str,
    root_dir: &Path,
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
                        deps.push(dep);
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
                    deps.push(dep);
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

