use anyhow::{Context, Result};
use std::process::Command;
use tempfile::TempDir;
use std::path::PathBuf;
use crate::parse;

// attempt to run apt-get -s upgrade inside a given chroot_path
pub fn run_apt_sim_in_chroot(chroot_path: &str) -> Result<String> {
    // Use 'chroot' + /bin/sh -c 'apt-get -s upgrade' (requires root)
    let out = Command::new("chroot")
        .arg(chroot_path)
        .arg("/bin/bash")
        .arg("-c")
        .arg("apt-get -s upgrade")
        .output()
        .context("failed to spawn chroot apt-get")?;

    if !out.status.success() {
        return Err(anyhow::anyhow!(
            "apt simulation failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

// Create disposable debootstrap rootfs for a given release (e.g., ubuntu focal)
pub fn create_debootstrap_root(release: &str) -> Result<TempDir> {
    // require debootstrap installed
    let temp = TempDir::new()?;
    let root = temp.path().to_string_lossy().to_string();
    let status = Command::new("debootstrap")
        .arg("--variant=minbase")
        .arg(release)
        .arg(&root)
        .arg("http://archive.ubuntu.com/ubuntu")
        .status()
        .context("failed to start debootstrap")?;

    if !status.success() {
        return Err(anyhow::anyhow!("debootstrap failed"));
    }
    Ok(temp)
}

// high-level: create disposable rootfs, run apt simulation, return output
pub fn simulate_with_debootstrap(release: &str) -> Result<String> {
    let tempdir = create_debootstrap_root(release)?;
    let root = tempdir.path().to_string_lossy().to_string();

    // mount /proc and /sys if necessary (skipped in this demo)
    let out = run_apt_sim_in_chroot(&root)?;
    // tempdir will be removed when drops
    Ok(out)
}

// Parse apt-get simulation to Simulation (parsing util in parse.rs)
pub fn parse_apt_output(output: &str) -> crate::Simulation {
    parse::parse_apt_simulation(output)
}
