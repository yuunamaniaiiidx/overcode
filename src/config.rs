use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use std::fs;
use std::io::Write;
use log::info;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub driver_patterns: Vec<MappingEntry>,
    #[serde(default)]
    pub mock_patterns: Vec<MappingEntry>,
    pub command: Option<CommandConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MappingEntry {
    pub pattern: String,
    #[serde(rename = "testcase")]
    pub testcase: String,
    #[serde(default)]
    pub mount_path: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CommandConfig {
    pub test: Option<RunTestConfig>,
    pub run: Option<RunTestConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ReplaceRule {
    pub pattern: String,
    pub replace: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RunTestConfig {
    pub command: String,
    pub args: Vec<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub replace_rule: Vec<ReplaceRule>,
}

impl Config {
    pub fn load(config_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;
        
        Ok(config)
    }

    fn get_template_content() -> &'static str {
        r#"# overcode.toml configuration file
"#
    }

    pub fn init_config(root_dir: &Path) -> Result<()> {
        let config_path = root_dir.join("overcode.toml");

        if config_path.exists() {
            info!("設定ファイルは既に存在します: {:?}", config_path);
            return Ok(());
        }

        info!("設定ファイルを作成します: {:?}", config_path);
        let template = Self::get_template_content();
        
        let mut file = fs::File::create(&config_path)
            .with_context(|| format!("Failed to create config file: {:?}", config_path))?;
        
        file.write_all(template.as_bytes())
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        info!("設定ファイルを作成しました: {:?}", config_path);
        Ok(())
    }
}

