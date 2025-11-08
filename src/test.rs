use anyhow::{Context, Result};
use std::process::Command;
use std::path::Path;

/// LD_PRELOADを使用してテストコマンドを実行する
pub fn run_test_command(root_dir: &Path, args: &[String]) -> Result<()> {
    if args.is_empty() {
        anyhow::bail!("テストコマンドが指定されていません。例: cargo run test cargo test");
    }

    // 現在のディレクトリを取得
    let current_dir = root_dir
        .canonicalize()
        .context("Failed to canonicalize root directory")?;

    // 共有ライブラリのパスを取得
    // target/debug または target/release から検索
    let lib_name = "libovercode_preload.so";
    let debug_lib = current_dir.join("target/debug").join(lib_name);
    let release_lib = current_dir.join("target/release").join(lib_name);
    
    let lib_path = if release_lib.exists() {
        release_lib
    } else if debug_lib.exists() {
        debug_lib
    } else {
        // ライブラリが存在しない場合はビルドを試みる
        println!("共有ライブラリが見つかりません。ビルドを試みます...");
        let build_status = Command::new("cargo")
            .arg("build")
            .current_dir(&current_dir)
            .status()
            .context("Failed to execute cargo build")?;
        
        if !build_status.success() {
            anyhow::bail!("共有ライブラリのビルドに失敗しました");
        }
        
        // ビルド後、再度パスを確認
        if debug_lib.exists() {
            debug_lib
        } else {
            anyhow::bail!("共有ライブラリのビルド後もファイルが見つかりません: {}", lib_name);
        }
    };

    // 共有ライブラリの絶対パスを取得
    let lib_path_abs = lib_path
        .canonicalize()
        .context("Failed to canonicalize library path")?;

    // コマンドを構築（LD_PRELOAD環境変数を設定）
    let mut cmd = Command::new(&args[0]);
    cmd.args(&args[1..]);
    cmd.current_dir(&current_dir);
    cmd.envs(std::env::vars());
    cmd.env("LD_PRELOAD", lib_path_abs.to_string_lossy().as_ref());

    // コマンドを実行（標準入出力を継承）
    let status = cmd
        .status()
        .context("Failed to execute test command")?;

    if !status.success() {
        anyhow::bail!("テストコマンドの実行に失敗しました: {}", status);
    }

    Ok(())
}

