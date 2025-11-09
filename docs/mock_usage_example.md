# モックを使用したテストの自動化手順

このドキュメントは、`podman_image_download/mock/podman_image.rs` モックを使用してテストを実行する手順を説明します。

## 概要

`test_pull_image_fails_without_internet_connection` テストは、インターネット接続がない環境をシミュレートする必要があります。このテストを通すために、実際の `podman_image_download.rs` をモック実装に一時的に差し替えます。

## 手動で行った操作

### 1. モックファイルの作成

モックファイル `src/podman_image_download/mock/podman_image.rs` を作成しました。このファイルには、常にエラーを返す `pull_image` 関数の実装が含まれています。

```rust
use anyhow::{Result, bail};

/// モック実装: インターネット接続がない想定でpull_imageを実装
pub fn pull_image(image: &str) -> Result<()> {
    // インターネット接続がない想定で、常にエラーを返す
    bail!(
        "Failed to pull image: {}. Error: network connection unavailable (mock)",
        image
    );
}
```

### 2. テスト実行前の準備

テストを実行する前に、以下の操作を行いました：

1. **元のファイルをバックアップ**
   ```bash
   cp src/podman_image_download.rs src/podman_image_download.rs.bak
   ```

2. **モックファイルの内容を元のファイルにコピー**
   ```bash
   cp src/podman_image_download/mock/podman_image.rs src/podman_image_download.rs
   ```

### 3. テストの実行

モックに差し替えた状態でテストを実行：

```bash
cargo test test_pull_image_fails_without_internet_connection
```

### 4. テスト後の復元

テスト実行後、元のファイルを復元：

```bash
mv src/podman_image_download.rs.bak src/podman_image_download.rs
```

## 自動化すべき操作

このツール（overcode）の目的は、以下の操作を自動化することです：

1. **モックファイルの検出**
   - `mock_patterns` 設定に基づいてモックファイルを検出
   - パターン: `(.+)/mock/.+.(.+)` → 解決先: `$1.$2`
   - 例: `podman_image_download/mock/podman_image.rs` → `podman_image_download.podman_image`

2. **テスト実行前の処理**
   - 対象の実装ファイル（`podman_image_download.rs`）をバックアップ
   - モックファイルの内容を実装ファイルにコピー

3. **テストの実行**
   - 指定されたテストを実行

4. **テスト後の処理**
   - バックアップから元の実装ファイルを復元

## ファイル構造

```
src/
├── podman_image_download.rs          # 実装ファイル（テスト時にモックに差し替えられる）
└── podman_image_download/
    └── mock/
        └── podman_image.rs           # モック実装
```

## テストファイル

テストは `src/podman_image/driver/podman_image_download.rs` に定義されています：

```rust
#[test]
fn test_pull_image_fails_without_internet_connection() {
    // インターネットに接続していない想定のシナリオ
    let result = podman_image_download::pull_image("docker.io/library/ubuntu:22.04");
    
    // エラーが返されることを期待
    assert!(
        result.is_err(),
        "pull_image should return an error when internet connection is unavailable and image is not local"
    );
    
    // エラーメッセージが空でないことを確認
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
    }
}
```

## 期待される動作

モックを使用した場合：
- ✅ テストが成功する（エラーが返される）
- ✅ エラーメッセージが空でない
- ✅ インターネット接続がない環境をシミュレートできる

実装ファイルを使用した場合（インターネット接続がある環境）：
- ❌ テストが失敗する（イメージのpullが成功してしまう）

## 設定ファイル

`overcode.toml` の `mock_patterns` 設定：

```toml
[[mock_patterns]]
pattern = "(.+)/mock/.+.(.+)"
resolution = "$1.$2"
```

この設定により、`podman_image_download/mock/podman_image.rs` が `podman_image_download.podman_image` として解決されます。

