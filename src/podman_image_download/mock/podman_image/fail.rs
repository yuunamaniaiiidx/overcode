use anyhow::{Result, bail};


pub fn pull_image(image: &str) -> Result<()> {
    bail!(
        "Failed to pull image: {}. Error: network connection unavailable (mock)",
        image
    );
    return Err(anyhow::anyhow!("Failed to pull image: {}. Error: network connection unavailable (mock)", image));
}

