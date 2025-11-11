#[cfg(test)]
mod tests {
    use crate::podman_image_download;
    #[test]
    fn test_pull_image_fails_without_internet_connection() {
        // インターネットに接続していない想定のシナリオ
        // 存在するイメージ名を指定するが、ローカルには存在せず、インターネット接続もないため失敗する
        // 
        // 注意: このテストは実際に失敗することを期待している
        // 現在の実装では、インターネット接続がない場合でも適切にエラーを返すべきだが、
        // 実際の環境ではネットワークエラーが発生する可能性がある
        
        // 存在するイメージ名（ただしローカルには存在しない想定）
        // インターネット接続がない環境では、このpullは失敗するはず
        let result = podman_image_download::pull_image("docker.io/library/ubuntu:22.04");
        
        // インターネット接続がない場合、エラーが返されることを期待
        // ただし、このテストは実際の環境に依存するため、失敗する可能性がある
        // 
        // 期待される動作:
        // - インターネット接続がない場合、pull_imageはエラーを返すべき
        // - エラーメッセージにはネットワーク関連の情報が含まれるべき
        assert!(
            result.is_err(),
        );
    }
}   