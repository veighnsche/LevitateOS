use anyhow::Result;
use std::env;
use std::path::Path;
use std::process::Command;

/// Applies the same PATH/LD_LIBRARY_PATH/OVMF_PATH environment wiring as the repo `justfile`.
pub fn apply_to_command(cmd: &mut Command, repo_root: &Path) -> Result<()> {
    let tools = crate::util::repo::tools_prefix(repo_root)?;

    let usr_bin = tools.join("usr/bin");
    let usr_libexec = tools.join("usr/libexec");
    let ld_library_path = tools.join("usr/lib64");
    let ovmf = crate::util::repo::ovmf_path(repo_root)?;

    let existing_path = env::var_os("PATH").unwrap_or_default();
    let mut paths = Vec::new();
    paths.push(usr_bin);
    paths.push(usr_libexec);
    paths.extend(env::split_paths(&existing_path));

    cmd.env("PATH", env::join_paths(paths)?);
    cmd.env("LD_LIBRARY_PATH", ld_library_path);
    cmd.env("OVMF_PATH", ovmf);
    Ok(())
}
