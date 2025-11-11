use anyhow::{Result, bail};

/// モック実装: インターネット接続がない想定でpull_imageを実装
/// 
/// このモックは、test_pull_image_fails_without_internet_connectionテストが
/// 通るように、常にエラーを返すpull_image関数を提供します。
/// 
/// 使用方法:
/// podman_image_download.rsを手動でこのモックに差し替えることで、
/// インターネット接続がない環境をシミュレートできます。

/// 指定されたイメージをダウンロード（モック実装）
/// 
/// このモック実装は、インターネット接続がない想定で常にエラーを返します。
/// test_pull_image_fails_without_internet_connectionテストが通るように設計されています。
pub fn pull_image(image: &str) -> Result<()> {
    // インターネット接続がない想定で、常にエラーを返す
    bail!(
        "Failed to pull image: {}. Error: network connection unavailable (mock)",
        image
    );
    return Err(anyhow::anyhow!("Failed to pull image: {}. Error: network connection unavailable (mock)", image));
}

