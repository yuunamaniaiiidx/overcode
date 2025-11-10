#[cfg(test)]
mod tests {
    use crate::podman_image_download;

    /// podman_image.rsがpodman_image_downloadに期待する動作をテストする
    /// 
    /// podman_image::ensure_images関数は以下の動作をpodman_image_downloadに期待している:
    /// 1. podman_image_download::pull_image関数が存在する
    /// 2. pull_image関数が&str型の引数を受け取る
    /// 3. pull_image関数がResult<()>を返す
    /// 4. 成功時はOk(())を返す
    /// 5. 失敗時はanyhow::Errorを返す（?演算子で処理できる）

    #[test]
    fn test_pull_image_accepts_str_and_returns_result() {
        // pull_image関数が&str型の引数を受け取り、Result<()>を返すことを確認
        // podman_image.rsの実際の使用パターンを再現
        // let image_name = &image_entry.name;  // image_entry.nameはString
        // podman_image_download::pull_image(image_name)?;
        let image_entry_name = String::from("docker.io/library/ubuntu:latest");
        let image_name: &str = &image_entry_name;
        let result = podman_image_download::pull_image(image_name);
        
        // pull_imageがOk(())を返すことを確認
        assert!(result.is_ok(), "pull_image should return Ok(())");
        
        // Result<()>であることを確認（成功時はOk(())）
        assert_eq!(result.unwrap(), ());
    }

    #[test]
    fn test_pull_image_can_be_used_with_question_mark() {
        // pull_image関数が?演算子で使用できることを確認
        // これはResult<(), anyhow::Error>を返すことを意味する
        
        fn test_function() -> anyhow::Result<()> {
            // podman_image.rsと同じパターンで使用
            podman_image_download::pull_image("docker.io/library/ubuntu:latest")?;
            Ok(())
        }
        
        // コンパイルが通れば、?演算子で使用できることが確認される
        let result = test_function();
        assert!(result.is_ok(), "test_function should return Ok(())");
    }

    #[test]
    fn test_pull_image_usage_pattern_matches_podman_image_rs() {
        // podman_image.rsの実際の使用パターンを完全に再現
        // for image_entry in &config.images {
        //     let image_name = &image_entry.name;
        //     ...
        //     podman_image_download::pull_image(image_name)?;
        // }
        
        struct ImageEntry {
            name: String,
        }
        
        let images = vec![
            ImageEntry { name: String::from("docker.io/library/ubuntu:latest") },
        ];
        
        fn process_images(images: &[ImageEntry]) -> anyhow::Result<()> {
            for image_entry in images {
                let image_name = &image_entry.name;
                // podman_image.rsと同じパターン
                podman_image_download::pull_image(image_name)?;
            }
            Ok(())
        }
        
        // コンパイルが通れば、podman_image.rsと同じパターンで使用できることが確認される
        let result = process_images(&images);
        assert!(result.is_ok(), "process_images should return Ok(())");
    }
}

