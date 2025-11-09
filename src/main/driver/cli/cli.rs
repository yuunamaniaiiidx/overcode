#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::cli::{Cli, Command};

    #[test]
    fn test_cli_parse() {
        // Cli::parse()はenv::args()に依存するため、実際のコマンドライン引数を使用する
        // このテストは、関数が呼び出し可能であることを確認する
        // 実際の動作確認は統合テストで行う
        let _result = Cli::parse();
        // 関数が呼び出し可能であることを確認（エラーでも成功でも、関数が実行されることを確認）
    }

    #[test]
    fn test_command_enum() {
        // Command enumのテスト
        let init = Command::Init;
        let test = Command::Test;
        let run = Command::Run;
        
        // PartialEqトレイトのテスト
        assert_eq!(init, Command::Init);
        assert_eq!(test, Command::Test);
        assert_eq!(run, Command::Run);
        
        // Debugトレイトのテスト（フォーマットが成功することを確認）
        let init_str = format!("{:?}", init);
        let test_str = format!("{:?}", test);
        let run_str = format!("{:?}", run);
        assert!(!init_str.is_empty());
        assert!(!test_str.is_empty());
        assert!(!run_str.is_empty());
    }

    #[test]
    fn test_cli_structure() {
        // Cli構造体のテスト
        let cli = Cli {
            command: Command::Init,
            root_dir: PathBuf::from("/tmp"),
            extra_args: vec![],
        };
        
        // 構造体のフィールドが正しく設定されていることを確認
        assert_eq!(cli.command, Command::Init);
        assert_eq!(cli.root_dir, PathBuf::from("/tmp"));
        assert_eq!(cli.extra_args.len(), 0);
        
        // Debugトレイトのテスト（フォーマットが成功することを確認）
        let cli_str = format!("{:?}", cli);
        assert!(!cli_str.is_empty());
    }
}

