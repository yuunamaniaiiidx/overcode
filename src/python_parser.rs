use regex::Regex;
use std::path::Path;

pub fn extract_dependencies(
    file_path: &Path,
    file_content: &str,
    root_dir: &Path,
) -> anyhow::Result<Vec<String>> {
    let mut deps = Vec::new();
    
    // import文のパターン
    let import_patterns = vec![
        // import module
        Regex::new(r"^import\s+([a-zA-Z_][a-zA-Z0-9_.]*)").unwrap(),
        // from module import ...
        Regex::new(r"^from\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s+import").unwrap(),
    ];

    // 標準ライブラリの主要モジュール（簡易的な判定）
    let stdlib_modules = [
        "os", "sys", "json", "re", "datetime", "collections", "itertools",
        "functools", "operator", "pathlib", "shutil", "subprocess", "threading",
        "multiprocessing", "asyncio", "urllib", "http", "email", "csv", "xml",
        "sqlite3", "hashlib", "base64", "uuid", "random", "math", "statistics",
        "decimal", "fractions", "array", "struct", "pickle", "copy", "pprint",
        "logging", "warnings", "traceback", "inspect", "types", "typing",
        "abc", "dataclasses", "enum", "contextlib", "unittest", "doctest",
        "pdb", "profile", "timeit", "dis", "gc", "weakref", "atexit",
    ];

    for line in file_content.lines() {
        let line = line.trim();
        
        // コメント行はスキップ
        if line.starts_with('#') {
            continue;
        }

        for pattern in &import_patterns {
            if let Some(captures) = pattern.captures(line) {
                if let Some(module) = captures.get(1) {
                    let module_name = module.as_str();
                    
                    // 標準ライブラリかチェック（最初の部分のみ）
                    let first_part = module_name.split('.').next().unwrap_or("");
                    if stdlib_modules.contains(&first_part) {
                        continue;
                    }

                    // 相対インポート（. で始まる）
                    if module_name.starts_with('.') {
                        let dep_path = resolve_relative_import(
                            file_path,
                            module_name,
                            root_dir,
                        )?;
                        if let Some(dep) = dep_path {
                            deps.push(dep);
                        }
                    } else {
                        // 絶対インポート - ローカルファイルかチェック
                        let dep_path = resolve_absolute_import(
                            file_path,
                            module_name,
                            root_dir,
                        )?;
                        if let Some(dep) = dep_path {
                            deps.push(dep);
                        }
                    }
                }
            }
        }
    }

    // 重複を除去
    deps.sort();
    deps.dedup();
    
    Ok(deps)
}

fn resolve_relative_import(
    file_path: &Path,
    module: &str,
    root_dir: &Path,
) -> anyhow::Result<Option<String>> {
    let file_dir = file_path.parent().unwrap_or(Path::new("."));
    let mut current = file_dir.to_path_buf();
    
    // . の数を数える
    let dots = module.chars().take_while(|&c| c == '.').count();
    let module_part = &module[dots..];
    
    // 親ディレクトリに移動
    for _ in 0..dots {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            return Ok(None);
        }
    }
    
    // モジュールパスを解決
    if module_part.is_empty() {
        return Ok(None);
    }
    
    let module_path = module_part.replace('.', "/");
    let candidate = current.join(&module_path);
    
    // .py ファイルか __init__.py を含むディレクトリを探す
    let py_file = candidate.with_extension("py");
    let init_dir = candidate.join("__init__.py");
    
    if py_file.exists() && py_file.starts_with(root_dir) {
        if let Ok(rel) = py_file.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    if init_dir.exists() && init_dir.starts_with(root_dir) {
        if let Ok(rel) = candidate.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    Ok(None)
}

fn resolve_absolute_import(
    file_path: &Path,
    module: &str,
    root_dir: &Path,
) -> anyhow::Result<Option<String>> {
    // まず、ファイルと同じディレクトリから探す
    let file_dir = file_path.parent().unwrap_or(Path::new("."));
    let module_path = module.replace('.', "/");
    
    // .py ファイル
    let py_file = file_dir.join(&module_path).with_extension("py");
    if py_file.exists() && py_file.starts_with(root_dir) {
        if let Ok(rel) = py_file.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    // __init__.py を含むディレクトリ
    let init_dir = file_dir.join(&module_path).join("__init__.py");
    if init_dir.exists() && init_dir.starts_with(root_dir) {
        let dir_path = init_dir.parent().unwrap();
        if let Ok(rel) = dir_path.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    // ルートディレクトリからも探す
    let py_file = root_dir.join(&module_path).with_extension("py");
    if py_file.exists() {
        if let Ok(rel) = py_file.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    let init_dir = root_dir.join(&module_path).join("__init__.py");
    if init_dir.exists() {
        let dir_path = init_dir.parent().unwrap();
        if let Ok(rel) = dir_path.strip_prefix(root_dir) {
            return Ok(Some(rel.to_string_lossy().to_string()));
        }
    }
    
    Ok(None)
}

