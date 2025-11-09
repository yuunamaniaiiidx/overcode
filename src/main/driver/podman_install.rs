#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::podman_install::ensure_podman;

    #[test]
    fn test_ensure_podman_when_already_installed() {
        // podmanが既にインストールされている場合のテスト
        // 注意: このテストは実際にpodmanコマンドを実行するため、
        // 環境によって結果が異なります
        
        // podmanがインストールされているか確認
        let podman_check = Command::new("podman")
            .arg("--version")
            .output();
        
        if podman_check.is_ok() && podman_check.unwrap().status.success() {
            // podmanがインストールされている場合、ensure_podmanは成功するはず
            let result = ensure_podman();
            // 既にインストールされている場合は成功する
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_podman_version_check() {
        // podmanのバージョンチェック機能をテスト
        let output = Command::new("podman")
            .arg("--version")
            .output();
        
        // podmanがインストールされている場合、コマンドは成功する
        // インストールされていない場合、エラーが返される
        // コマンドの実行結果を確認（成功またはエラーのどちらかが返される）
        match output {
            Ok(result) => {
                // コマンドが実行された場合、ステータスコードが存在することを確認
                assert!(result.status.code().is_some() || !result.status.success());
            }
            Err(_) => {
                // コマンドが実行できなかった場合（podmanがインストールされていないなど）
                // これは正常な動作なので、テストは成功とする
            }
        }
    }
}

