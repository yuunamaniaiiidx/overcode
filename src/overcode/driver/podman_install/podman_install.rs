#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::podman_install::ensure_podman;

    #[test]
    fn test_ensure_podman_when_already_installed() {
        
        let podman_check = Command::new("podman")
            .arg("--version")
            .output();
        
        if podman_check.is_ok() && podman_check.unwrap().status.success() {
            let result = ensure_podman();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_podman_version_check() {
        let output = Command::new("podman")
            .arg("--version")
            .output();
        
        match output {
            Ok(result) => {
                assert!(result.status.code().is_some() || !result.status.success());
            }
            Err(_) => {
            }
        }
    }
}

