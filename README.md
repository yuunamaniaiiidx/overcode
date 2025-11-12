# overcode

overcodeは、Rustプロジェクトのテストと実行を管理するためのツールです。パターンマッチングに基づいてドライバーファイルとモックファイルを自動的に検出し、Podmanコンテナ内でテストや実行を行います。

## 機能

- **設定ファイルの初期化**: `overcode.toml`設定ファイルを自動生成
- **テスト実行**: ドライバーパターンに基づいてテストを自動実行
- **プロジェクト実行**: 設定に基づいてプロジェクトを実行
- **Podman統合**: コンテナ内でのコマンド実行をサポート

## インストール

### ビルド方法

```bash
cd overcode
cargo build --release
```

ビルドされたバイナリは `target/release/overcode` に配置されます。

## 使用方法

### 初期化

プロジェクトディレクトリで設定ファイルを初期化します：

```bash
overcode init
```

または、特定のパスに設定ファイルを作成：

```bash
overcode init --config /path/to/overcode.toml
```

### テスト実行

設定ファイルに基づいてテストを実行します：

```bash
overcode test
```

または、特定の設定ファイルを指定：

```bash
overcode test --config /path/to/overcode.toml
```

### プロジェクト実行

プロジェクトを実行します：

```bash
overcode run
```

追加の引数を渡す場合：

```bash
overcode run -- extra-args-here
```

## 設定ファイル

`overcode.toml`ファイルでプロジェクトの設定を行います。

### 設定例

```toml
[[driver_patterns]]
pattern = "src/([^/]+)/driver/([^/]+)/([^/]+)\\.rs"
testcase = "$2_$3"

[[mock_patterns]]
pattern = "src/([^/]+)/mock/([^/]+)/([^/]+)\\.rs"
testcase = "$1_$3"
mount_path = "src/$1.rs"

[command.test]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["test", "--manifest-path", "Cargo.toml", "{driver_file}"]

replace_rule = [
  { pattern = "src/([^/]+)/driver/([^/]+)/([^/]+)\\.rs", replace = "$1::driver_$2_$3" },
]

[command.run]
image = "docker.io/library/rust:latest"
command = "cargo"
args = ["run", "--manifest-path", "Cargo.toml"]
```

### 設定項目

- **driver_patterns**: ドライバーファイルのパターンを定義
  - `pattern`: ファイルパスにマッチする正規表現
  - `testcase`: テストケース名の生成パターン
- **mock_patterns**: モックファイルのパターンを定義
  - `pattern`: ファイルパスにマッチする正規表現
  - `testcase`: テストケース名の生成パターン
  - `mount_path`: マウント先のパス（オプション）
- **command.test**: テスト実行コマンドの設定
  - `image`: 使用するPodmanイメージ（オプション）
  - `command`: 実行するコマンド
  - `args`: コマンドの引数
  - `replace_rule`: パターン置換ルール（オプション）
- **command.run**: 実行コマンドの設定
  - `image`: 使用するPodmanイメージ（オプション）
  - `command`: 実行するコマンド
  - `args`: コマンドの引数

## 依存関係

- **Podman**: コンテナ実行に必要（自動インストール機能あり）

## ライセンス

MIT License - 詳細は [LICENSE](LICENSE) ファイルを参照してください。

