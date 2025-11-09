use std::path::Path;

/// podman runコマンドのマウント引数を構築する
/// 
/// # Arguments
/// * `root_dir` - マウントするルートディレクトリ
/// 
/// # Returns
/// podman runコマンドのマウント引数ベクター（-v {host_path}:{container_path}）
pub fn build_mount_args(root_dir: &Path) -> Vec<String> {
    let root_dir_str = root_dir.display().to_string();
    
    // -v {root_dir}:{root_dir}
    vec![
        "-v".to_string(),
        format!("{}:{}", root_dir_str, root_dir_str),
    ]
}

