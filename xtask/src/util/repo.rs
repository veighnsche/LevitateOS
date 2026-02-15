use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub fn repo_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .context("xtask is expected at <repo>/xtask")
}

pub fn tools_prefix(root: &Path) -> PathBuf {
    root.join("leviso/downloads/.tools")
}

pub fn ovmf_path(root: &Path) -> PathBuf {
    tools_prefix(root).join("usr/share/edk2/ovmf/OVMF_CODE.fd")
}
