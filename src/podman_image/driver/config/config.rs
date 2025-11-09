#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;
    use crate::config::Config;

    /// podman_image.rsがconfigに期待する動作をテストする
    /// 
    /// podman_image::ensure_images関数は以下の動作をconfigに期待している:
    /// 1. Config::load_from_root(root_dir)がResult<Config>を返す
    /// 2. config.imagesがVec<ImageEntry>である
    /// 3. config.images.is_empty()が動作する
    /// 4. config.images.len()が動作する
    /// 5. config.imagesをイテレートできる
    /// 6. 各image_entryがnameフィールドを持つ
    /// 7. image_entry.nameが&strとして取得できる

    #[test]
    fn test_config_load_from_root_returns_result() {
        // Config::load_from_rootがResult<Config>を返すことを確認
        let temp_dir = TempDir::new().unwrap();
        let result = Config::load_from_root(temp_dir.path());
        
        // Resultが返されることを確認
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_images_is_vec_image_entry() {
        // config.imagesがVec<ImageEntry>であることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // imagesがVec<ImageEntry>であることを確認
        assert_eq!(config.images.len(), 1);
    }

    #[test]
    fn test_config_images_is_empty_works() {
        // config.images.is_empty()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルが存在しない場合、空のimagesが返される
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        assert!(config.images.is_empty());
        
        // 明示的に空のimagesを指定した場合
        let config_path = temp_dir.path().join("overcode.toml");
        let toml_content = r#"
# imagesセクションなし
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        assert!(config.images.is_empty());
    }

    #[test]
    fn test_config_images_len_works() {
        // config.images.len()が動作することを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"

[[images]]
name = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // len()が正しく動作することを確認
        assert_eq!(config.images.len(), 2);
    }

    #[test]
    fn test_config_images_can_be_iterated() {
        // config.imagesをイテレートできることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"

[[images]]
name = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // イテレートできることを確認
        let mut count = 0;
        for _image_entry in &config.images {
            count += 1;
        }
        assert_eq!(count, 2);
    }

    #[test]
    fn test_image_entry_has_name_field() {
        // 各image_entryがnameフィールドを持つことを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // nameフィールドが存在することを確認
        assert_eq!(config.images[0].name, "docker.io/library/ubuntu:latest");
    }

    #[test]
    fn test_image_entry_name_is_str() {
        // image_entry.nameが&strとして取得できることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // nameが&strとして取得できることを確認
        for image_entry in &config.images {
            let image_name: &str = &image_entry.name;
            assert_eq!(image_name, "docker.io/library/ubuntu:latest");
        }
    }

    #[test]
    fn test_config_load_from_root_with_missing_file_returns_empty_images() {
        // 設定ファイルが存在しない場合、空のimagesが返されることを確認
        // これはpodman_image.rsが早期リターンするために必要な動作
        let temp_dir = TempDir::new().unwrap();
        
        // 設定ファイルを作成しない
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // 空のimagesが返されることを確認
        assert!(config.images.is_empty());
    }

    #[test]
    fn test_config_images_multiple_entries() {
        // 複数のイメージエントリが正しく読み込まれることを確認
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"

[[images]]
name = "docker.io/library/rust:latest"

[[images]]
name = "docker.io/library/python:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // すべてのイメージが読み込まれることを確認
        assert_eq!(config.images.len(), 3);
        assert_eq!(config.images[0].name, "docker.io/library/ubuntu:latest");
        assert_eq!(config.images[1].name, "docker.io/library/rust:latest");
        assert_eq!(config.images[2].name, "docker.io/library/python:latest");
    }

    #[test]
    fn test_config_images_iteration_with_name_access() {
        // podman_image.rsの実際の使用パターンを再現
        // for image_entry in &config.images {
        //     let image_name = &image_entry.name;
        //     ...
        // }
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("overcode.toml");
        
        let toml_content = r#"
[[images]]
name = "docker.io/library/ubuntu:latest"

[[images]]
name = "docker.io/library/rust:latest"
"#;
        fs::write(&config_path, toml_content).unwrap();
        
        let config = Config::load_from_root(temp_dir.path()).unwrap();
        
        // podman_image.rsと同じパターンでアクセスできることを確認
        let mut names = Vec::new();
        for image_entry in &config.images {
            let image_name = &image_entry.name;
            names.push(image_name.clone());
        }
        
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "docker.io/library/ubuntu:latest");
        assert_eq!(names[1], "docker.io/library/rust:latest");
    }
}

