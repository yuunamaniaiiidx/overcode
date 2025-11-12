use std::path::Path;

pub fn build_mount_args(root_dir: &Path) -> Vec<String> {
    let root_dir_str = root_dir.display().to_string();
    
    vec![
        "-v".to_string(),
        format!("{}:{}", root_dir_str, root_dir_str),
    ]
}

