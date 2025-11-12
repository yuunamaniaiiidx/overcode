#[cfg(test)]
mod tests {
    use crate::podman_image_download;


    #[test]
    fn test_pull_image_accepts_str_and_returns_result() {
        let result = podman_image_download::pull_image("docker.io/library/ubuntu:latest");
        
        assert!(result.is_ok());
        
        assert_eq!(result.unwrap(), ());
    }
}

