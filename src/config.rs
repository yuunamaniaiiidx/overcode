use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::fs;
use std::io::Write;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ignores: Vec<IgnoreEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IgnoreEntry {
    pub path: String,
}

pub struct IgnorePattern {
    pattern: String,
}

impl IgnorePattern {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }

    /// パスがこのパターンにマッチするかチェック
    /// gitignoreライクな動作:
    /// - パスの任意の部分にパターンが含まれるかチェック
    /// - ディレクトリ名やファイル名が一致するかチェック
    /// - ワイルドカード（*）をサポート
    pub fn matches(&self, path: &Path, root: &Path) -> bool {
        // 相対パスに変換
        let relative_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // パターンを正規表現に変換（シンプルな実装）
        let pattern = self.pattern.as_str();
        
        // パスの各コンポーネントをチェック
        for component in relative_path.components() {
            let component_str = component.as_os_str().to_string_lossy();
            
            // 完全一致
            if component_str == pattern {
                return true;
            }
            
            // ワイルドカードパターンのマッチング
            if self.matches_wildcard(&component_str, pattern) {
                return true;
            }
        }
        
        // パス全体の文字列としてもチェック
        let path_str = relative_path.to_string_lossy();
        if path_str.contains(pattern) {
            return true;
        }
        
        // ワイルドカードパターンでパス全体をチェック
        if self.matches_wildcard(&path_str, pattern) {
            return true;
        }
        
        false
    }

    /// シンプルなワイルドカードマッチング（*のみサポート）
    fn matches_wildcard(&self, text: &str, pattern: &str) -> bool {
        if !pattern.contains('*') {
            return false;
        }

        // パターンを*で分割
        let parts: Vec<&str> = pattern.split('*').collect();
        
        if parts.is_empty() {
            return false;
        }

        // 最初の部分で開始するかチェック
        if !parts[0].is_empty() && !text.starts_with(parts[0]) {
            return false;
        }

        // 最後の部分で終わるかチェック
        if parts.len() > 1 {
            let last_part = parts[parts.len() - 1];
            if !last_part.is_empty() && !text.ends_with(last_part) {
                return false;
            }
        }

        // 中間部分が順番に含まれているかチェック
        let mut search_start = 0;
        for (i, part) in parts.iter().enumerate() {
            if part.is_empty() {
                continue;
            }
            
            if i == 0 {
                search_start = part.len();
            } else if i == parts.len() - 1 {
                // 最後の部分は既にチェック済み
                break;
            } else {
                match text[search_start..].find(part) {
                    Some(pos) => {
                        search_start += pos + part.len();
                    }
                    None => {
                        return false;
                    }
                }
            }
        }

        true
    }
}

impl Config {
    /// overcode.tomlファイルを読み込む
    pub fn load(config_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;
        
        Ok(config)
    }

    /// ルートディレクトリから設定ファイルを探して読み込む
    pub fn load_from_root(root_dir: &Path) -> Result<Self> {
        let config_path = root_dir.join("overcode.toml");
        
        if !config_path.exists() {
            // 設定ファイルが存在しない場合は空の設定を返す
            return Ok(Config {
                ignores: Vec::new(),
            });
        }
        
        Self::load(&config_path)
    }

    /// ignoreパターンのリストを取得
    pub fn get_ignore_patterns(&self) -> Vec<IgnorePattern> {
        self.ignores
            .iter()
            .map(|entry| IgnorePattern::new(&entry.path))
            .collect()
    }

    /// 設定ファイルのテンプレート内容を返す
    fn get_template_content() -> &'static str {
        r#"[[ignores]]
path = ".git"
"#
    }

    /// 設定ファイルを初期化する（存在しない場合にテンプレートを作成）
    pub fn init_config(root_dir: &Path) -> Result<()> {
        let config_path = root_dir.join("overcode.toml");

        if config_path.exists() {
            println!("設定ファイルは既に存在します: {:?}", config_path);
            return Ok(());
        }

        println!("設定ファイルを作成します: {:?}", config_path);
        let template = Self::get_template_content();
        
        let mut file = fs::File::create(&config_path)
            .with_context(|| format!("Failed to create config file: {:?}", config_path))?;
        
        file.write_all(template.as_bytes())
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        println!("設定ファイルを作成しました: {:?}", config_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_ignore_pattern_exact_match() {
        let pattern = IgnorePattern::new(".git");
        let root = PathBuf::from("/project");
        let path = PathBuf::from("/project/.git/config");
        
        assert!(pattern.matches(&path, &root));
    }

    #[test]
    fn test_ignore_pattern_wildcard() {
        let pattern = IgnorePattern::new("*.log");
        let root = PathBuf::from("/project");
        let path = PathBuf::from("/project/src/app.log");
        
        assert!(pattern.matches(&path, &root));
    }

    #[test]
    fn test_ignore_pattern_directory() {
        let pattern = IgnorePattern::new("node_modules");
        let root = PathBuf::from("/project");
        let path = PathBuf::from("/project/src/node_modules/package.json");
        
        assert!(pattern.matches(&path, &root));
    }
}

