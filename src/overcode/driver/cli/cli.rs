#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::cli::{Cli, Command};

    #[test]
    fn test_cli_parse() {
        let _result = Cli::parse();
    }

    #[test]
    fn test_command_enum() {
        let init = Command::Init;
        let test = Command::Test;
        let run = Command::Run;
        
        assert_eq!(init, Command::Init);
        assert_eq!(test, Command::Test);
        assert_eq!(run, Command::Run);
        
        let init_str = format!("{:?}", init);
        let test_str = format!("{:?}", test);
        let run_str = format!("{:?}", run);
        assert!(!init_str.is_empty());
        assert!(!test_str.is_empty());
        assert!(!run_str.is_empty());
    }

    #[test]
    fn test_cli_structure() {
        let cli = Cli {
            command: Command::Init,
            root_dir: PathBuf::from("/tmp"),
            config_path: PathBuf::from("/tmp/overcode.toml"),
            extra_args: vec![],
        };
        
        assert_eq!(cli.command, Command::Init);
        assert_eq!(cli.root_dir, PathBuf::from("/tmp"));
        assert_eq!(cli.config_path, PathBuf::from("/tmp/overcode.toml"));
        assert_eq!(cli.extra_args.len(), 0);
        
        let cli_str = format!("{:?}", cli);
        assert!(!cli_str.is_empty());
    }
}

