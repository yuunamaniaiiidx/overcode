mod cli;
mod config;
mod overcode;
mod podman_image;
mod podman_image_download;
mod podman_install;
mod podman_mount;
mod run;
mod test;

fn main() -> anyhow::Result<()> {
    overcode::main()
}