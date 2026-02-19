use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};

pub fn repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask is expected at <repo>/xtask")
}

pub fn canonical_tools_install_command() -> &'static str {
    "recipe install --build-dir .artifacts/tools --recipes-path distro-builder/recipes qemu-deps"
}

pub fn tools_prefix(root: &Path) -> Result<PathBuf> {
    let centralized = root.join(".artifacts/tools/.tools");
    if !fs::metadata(&centralized)
        .with_context(|| format!("checking tools root at {}", centralized.display()))?
        .is_dir()
    {
        bail!(
            "missing canonical tools root: {}\nInstall with: {}",
            centralized.display(),
            canonical_tools_install_command()
        );
    }
    Ok(centralized)
}

pub fn ovmf_path(root: &Path) -> Result<PathBuf> {
    let tools_prefix = tools_prefix(root)?;
    Ok(tools_prefix.join("usr/share/edk2/ovmf/OVMF_CODE.fd"))
}
