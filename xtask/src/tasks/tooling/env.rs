use anyhow::Result;

pub fn run(shell: crate::cli::Shell) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let tools = crate::util::repo::tools_prefix(&root)?;

    let usr_bin = tools.join("usr/bin");
    let usr_libexec = tools.join("usr/libexec");
    let ld_library_path = tools.join("usr/lib64");
    let ovmf = crate::util::repo::ovmf_path(&root)?;

    // This is intentionally the same wiring as the justfile.
    // Keep it as pure string exports so users can `eval` it.
    let path_export = format!("{}:{}:$PATH", usr_bin.display(), usr_libexec.display());

    match shell {
        crate::cli::Shell::Bash | crate::cli::Shell::Sh => {
            println!("export PATH=\"{}\"", path_export);
            println!("export LD_LIBRARY_PATH=\"{}\"", ld_library_path.display());
            println!("export OVMF_PATH=\"{}\"", ovmf.display());
        }
    }

    Ok(())
}
