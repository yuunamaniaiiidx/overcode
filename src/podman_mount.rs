use std::path::Path;

/// podman runコマンドのマウント引数を構築する
/// 
/// # Arguments
/// * `root_dir` - マウントするルートディレクトリ
/// 
/// # Returns
/// podman runコマンドの引数ベクター（run, --rm, -v, -w など）
pub fn build_mount_args(root_dir: &Path) -> Vec<String> {
    let root_dir_str = root_dir.display().to_string();
    
    // podman run --rm -v {root_dir}:{root_dir} -w {root_dir}
    vec![
        "run".to_string(),
        "--rm".to_string(),
        "-v".to_string(),
        format!("{}:{}", root_dir_str, root_dir_str),
        "-w".to_string(),
        root_dir_str,
    ]
}

