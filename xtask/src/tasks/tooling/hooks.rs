use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn install() -> Result<()> {
    run(Mode::Install)
}

pub fn remove() -> Result<()> {
    run(Mode::Remove)
}

#[derive(Clone, Copy, Debug)]
enum Mode {
    Install,
    Remove,
}

fn run(mode: Mode) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let hook_source = root.join("tools/pre-commit-hook.sh");

    if !hook_source.is_file() {
        anyhow::bail!("Missing hook source: {}", hook_source.display());
    }

    match mode {
        Mode::Install => {
            eprintln!("Installing pre-commit hooks...");
            eprintln!();

            ensure_executable(&hook_source)?;

            let mut installed = 0usize;
            let mut skipped = 0usize;

            if let Some(hooks_dir) = hooks_dir_for_repo(&root)? {
                install_one(&hooks_dir, "(parent repo)", &hook_source)?;
                installed += 1;
            }

            for sub in rust_submodules() {
                let sub_path = root.join(sub);
                match hooks_dir_for_repo(&sub_path)? {
                    Some(hooks_dir) => {
                        install_one(&hooks_dir, sub, &hook_source)?;
                        installed += 1;
                    }
                    None => {
                        eprintln!("  miss  {:<30} (hooks dir not found)", sub);
                        skipped += 1;
                    }
                }
            }

            eprintln!();
            eprintln!("Installed: {installed}  Skipped: {skipped}");
            eprintln!();
            eprintln!("Hook runs: cargo fmt (auto-fix) + clippy + unit tests");
            eprintln!("Skip with: git commit --no-verify");
        }
        Mode::Remove => {
            eprintln!("Removing pre-commit hooks...");
            eprintln!();

            if let Some(hooks_dir) = hooks_dir_for_repo(&root)? {
                remove_one(&hooks_dir, "(parent repo)", &hook_source)?;
            }

            for sub in rust_submodules() {
                let sub_path = root.join(sub);
                if let Some(hooks_dir) = hooks_dir_for_repo(&sub_path)? {
                    remove_one(&hooks_dir, sub, &hook_source)?;
                }
            }

            eprintln!();
            eprintln!("Done.");
        }
    }

    Ok(())
}

fn rust_submodules() -> &'static [&'static str] {
    &[
        "AcornOS",
        "IuppiterOS",
        "distro-builder",
        "distro-spec",
        "leviso",
        "leviso-elf",
        "testing/cheat-guard",
        "testing/cheat-test",
        "testing/fsdbg",
        "testing/hardware-compat",
        "testing/install-tests",
        "testing/rootfs-tests",
        "tools/recchroot",
        "tools/recfstab",
        "tools/recinit",
        "tools/recipe",
        "tools/reciso",
        "tools/recqemu",
        "tools/recstrap",
        "tools/recuki",
    ]
}

fn hooks_dir_for_repo(repo_path: &Path) -> Result<Option<PathBuf>> {
    let dot_git = repo_path.join(".git");

    if dot_git.is_dir() {
        let hooks = dot_git.join("hooks");
        return Ok(hooks.is_dir().then_some(hooks));
    }

    if !dot_git.is_file() {
        return Ok(None);
    }

    let gitdir =
        fs::read_to_string(&dot_git).with_context(|| format!("Reading {}", dot_git.display()))?;
    let gitdir = gitdir
        .trim()
        .strip_prefix("gitdir: ")
        .map(str::trim)
        .unwrap_or(gitdir.trim());

    let hooks_dir = repo_path.join(gitdir).join("hooks");
    Ok(hooks_dir.is_dir().then_some(hooks_dir))
}

fn install_one(hooks_dir: &Path, label: &str, hook_source: &Path) -> Result<()> {
    let target = hooks_dir.join("pre-commit");
    let backup = hooks_dir.join("pre-commit.backup");

    if is_our_hook(&target, hook_source)? {
        eprintln!("  skip  {:<30} (already installed)", label);
        return Ok(());
    }

    // Back up non-symlink hooks before overwriting.
    if target.exists() && !target.is_symlink() {
        if backup.exists() {
            // Match shell-script behavior (mv overwrites): remove the old backup.
            fs::remove_file(&backup).with_context(|| format!("Removing {}", backup.display()))?;
        }
        fs::rename(&target, &backup).with_context(|| {
            format!(
                "Backing up existing hook {} -> {}",
                target.display(),
                backup.display()
            )
        })?;
        eprintln!(
            "  back  {:<30} (existing hook backed up to pre-commit.backup)",
            label
        );
    }

    if target.exists() || target.is_symlink() {
        fs::remove_file(&target).with_context(|| format!("Removing {}", target.display()))?;
    }

    symlink_file(hook_source, &target).with_context(|| {
        format!(
            "Creating symlink {} -> {}",
            target.display(),
            hook_source.display()
        )
    })?;
    eprintln!("  done  {:<30}", label);
    Ok(())
}

fn remove_one(hooks_dir: &Path, label: &str, hook_source: &Path) -> Result<()> {
    let target = hooks_dir.join("pre-commit");
    let backup = hooks_dir.join("pre-commit.backup");

    if is_our_hook(&target, hook_source)? {
        fs::remove_file(&target).with_context(|| format!("Removing {}", target.display()))?;
        if backup.is_file() {
            fs::rename(&backup, &target).with_context(|| {
                format!(
                    "Restoring backup {} -> {}",
                    backup.display(),
                    target.display()
                )
            })?;
            eprintln!("  rest  {:<30} (restored backup)", label);
        } else {
            eprintln!("  done  {:<30}", label);
        }
    } else {
        eprintln!("  skip  {:<30} (not our hook)", label);
    }

    Ok(())
}

fn is_our_hook(target: &Path, hook_source: &Path) -> Result<bool> {
    if !target.is_symlink() {
        return Ok(false);
    }

    let want = match fs::canonicalize(hook_source) {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };
    let got = match fs::canonicalize(target) {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };
    Ok(got == want)
}

fn ensure_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        // Ensure u+x (and keep existing bits).
        perms.set_mode(perms.mode() | 0o100);
        fs::set_permissions(path, perms)?;
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

fn symlink_file(src: &Path, dst: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(src, dst)?;
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        let _ = (src, dst);
        anyhow::bail!("Symlinks are only supported on unix platforms");
    }
}
