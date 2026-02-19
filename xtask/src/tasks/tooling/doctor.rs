use anyhow::{Result, bail};

pub fn run() -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let tools = crate::util::repo::tools_prefix(&root)?;
    let ovmf = crate::util::repo::ovmf_path(&root)?;

    let mut ok = true;

    if which::which("just").is_err() {
        eprintln!("[FAIL] missing `just` in PATH");
        ok = false;
    } else {
        eprintln!("[OK] just");
    }

    let want_dirs = [
        tools.join("usr/bin"),
        tools.join("usr/libexec"),
        tools.join("usr/lib64"),
    ];
    for d in want_dirs {
        if d.is_dir() {
            eprintln!("[OK] {}", d.display());
        } else {
            eprintln!("[FAIL] missing directory: {}", d.display());
            ok = false;
        }
    }

    if ovmf.is_file() {
        eprintln!("[OK] {}", ovmf.display());
    } else {
        eprintln!("[FAIL] missing OVMF firmware: {}", ovmf.display());
        ok = false;
    }

    if !ok {
        bail!("doctor checks failed");
    }
    Ok(())
}
