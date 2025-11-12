#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use tempfile::TempDir;
    use crate::podman_mount::build_mount_args;

    #[test]
    fn test_build_mount_args_with_simple_path() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();
        
        let args = build_mount_args(root_dir);
        
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-v");
        let mount_arg = format!("{}:{}", root_dir.display(), root_dir.display());
        assert_eq!(args[1], mount_arg);
    }

    #[test]
    fn test_build_mount_args_with_path_containing_spaces() {
        let temp_dir = TempDir::new().unwrap();
        let path_with_spaces = temp_dir.path().join("path with spaces");
        std::fs::create_dir_all(&path_with_spaces).unwrap();
        
        let args = build_mount_args(&path_with_spaces);
        
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-v");
        let mount_arg = format!("{}:{}", path_with_spaces.display(), path_with_spaces.display());
        assert_eq!(args[1], mount_arg);
    }

    #[test]
    fn test_build_mount_args_with_absolute_path() {
        let root_dir = PathBuf::from("/tmp/test");
        
        let args = build_mount_args(&root_dir);
        
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-v");
        let mount_arg = format!("{}:{}", root_dir.display(), root_dir.display());
        assert_eq!(args[1], mount_arg);
    }

    #[test]
    fn test_build_mount_args_mount_format() {
        let temp_dir = TempDir::new().unwrap();
        let root_dir = temp_dir.path();
        
        let args = build_mount_args(root_dir);
        
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-v");
        let mount_value = &args[1];
        let parts: Vec<&str> = mount_value.split(':').collect();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], parts[1]); 
        assert_eq!(parts[0], root_dir.display().to_string());
    }
}

