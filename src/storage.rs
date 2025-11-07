use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone)]
pub struct SourceEntry {
    pub paths: Vec<String>,
    pub hash: String,
    pub deps: Vec<String>,
}

pub struct Storage {
    overcode_dir: PathBuf,
}

impl Storage {
    pub fn new(root: &Path) -> anyhow::Result<Self> {
        let overcode_dir = root.join(".overcode");
        if !overcode_dir.exists() {
            fs::create_dir_all(&overcode_dir)?;
        }
        Ok(Self { overcode_dir })
    }

    pub fn load_meta(&self, hash: &str) -> anyhow::Result<Vec<SourceEntry>> {
        let meta_path = self.overcode_dir.join(format!("{}.toml", hash));
        if !meta_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&meta_path)?;
        let value: Value = toml::from_str(&content)?;

        let mut entries = Vec::new();
        if let Some(srcs) = value.get("srcs").and_then(|v| v.as_array()) {
            for src in srcs {
                if let Some(table) = src.as_table() {
                    let paths = table
                        .get("path")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();
                    let hash = table
                        .get("hash")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let deps = table
                        .get("deps")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default();

                    entries.push(SourceEntry { paths, hash, deps });
                }
            }
        }

        Ok(entries)
    }

    pub fn save_file(&self, hash: &str, content: &[u8]) -> anyhow::Result<()> {
        let file_path = self.overcode_dir.join(hash);
        if !file_path.exists() {
            let mut file = fs::File::create(&file_path)?;
            file.write_all(content)?;
        }
        Ok(())
    }

    pub fn save_meta(&self, hash: &str, entries: &[SourceEntry]) -> anyhow::Result<()> {
        let meta_path = self.overcode_dir.join(format!("{}.toml", hash));
        
        let mut toml_value = toml::map::Map::new();
        let mut srcs_array = Vec::new();

        for entry in entries {
            let mut src_table = toml::map::Map::new();
            src_table.insert("path".to_string(), Value::Array(
                entry.paths.iter().map(|p| Value::String(p.clone())).collect()
            ));
            src_table.insert("hash".to_string(), Value::String(entry.hash.clone()));
            src_table.insert("deps".to_string(), Value::Array(
                entry.deps.iter().map(|d| Value::String(d.clone())).collect()
            ));
            srcs_array.push(Value::Table(src_table));
        }

        toml_value.insert("srcs".to_string(), Value::Array(srcs_array));
        let toml_string = toml::to_string_pretty(&Value::Table(toml_value))?;
        fs::write(&meta_path, toml_string)?;
        Ok(())
    }

    pub fn get_all_known_hashes(&self) -> anyhow::Result<HashMap<String, Vec<SourceEntry>>> {
        let mut known = HashMap::new();
        
        if !self.overcode_dir.exists() {
            return Ok(known);
        }

        for entry in fs::read_dir(&self.overcode_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "toml" {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        let entries = self.load_meta(stem)?;
                        if !entries.is_empty() {
                            known.insert(stem.to_string(), entries);
                        }
                    }
                }
            }
        }

        Ok(known)
    }
}

