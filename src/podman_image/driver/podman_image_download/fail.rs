#[cfg(test)]
mod tests {
    use crate::podman_image_download;
    #[test]
    fn test_pull_image_fails_without_internet_connection() {
        
        let result = podman_image_download::pull_image("docker.io/library/ubuntu:22.04");
        
        assert!(
            result.is_err(),
        );
    }
}   