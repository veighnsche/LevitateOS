use anyhow::{Context, Result};
use std::path::Path;

pub fn run(rebuild_even_if_verified: bool, autofix: super::common::AutoFixOptions) -> Result<()> {
    let root = crate::util::repo::repo_root()?;

    eprintln!("[info] Repo: {}", root.display());

    let targets = [
        super::common::target_for(crate::cli::Distro::Leviso),
        super::common::target_for(crate::cli::Distro::AcornOS),
        super::common::target_for(crate::cli::Distro::IuppiterOS),
        super::common::target_for(crate::cli::Distro::RalphOS),
    ];

    let need_build = rebuild_even_if_verified
        || targets
            .iter()
            .any(|t| !super::common::kernel_is_built(&root, t));
    if need_build {
        // Policy is about preventing accidental "start a laptop-melter at noon".
        // If you're inside the allowed window at start, let the full run complete.
        super::common::enforce_build_hours()?;
    }

    // We intentionally run the shared kernel recipe directly (not the per-distro CLIs)
    // so the "nightly build-all" path is identical across all distros.
    let recipe_bin = if need_build {
        Some(super::common::build_recipe_bin(&root).context("Failed to build recipe binary")?)
    } else {
        None
    };

    eprintln!("[info] Kernel targets (from distro-spec):");
    for t in targets.iter() {
        eprintln!(
            "  {}: {}{}",
            t.distro_id, t.kernel.version, t.kernel.localversion
        );
    }

    for t in targets.iter() {
        maybe_build(root.as_path(), rebuild_even_if_verified, t, || {
            let recipe_bin = recipe_bin
                .as_ref()
                .context("recipe binary missing (internal error)")?;
            super::common::build_kernel_via_recipe(
                recipe_bin.as_path(),
                &root,
                t.distro_id,
                rebuild_even_if_verified,
                t.kernel,
                t.module_install_path,
                &autofix,
            )
        })?;
    }

    Ok(())
}

fn maybe_build<F>(
    root: &Path,
    rebuild_even_if_verified: bool,
    t: &super::common::KernelTarget,
    build: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if !rebuild_even_if_verified && super::common::kernel_is_built(root, t) {
        eprintln!("[skip] {} kernel already built+verified", t.distro_id);
        return Ok(());
    }

    eprintln!("[step] Build kernel: {}", t.distro_id);
    if let Err(e) = build() {
        return Err(e).context(format!("Kernel build failed for {}", t.distro_id));
    }

    let rel = match super::common::verify_one(root, t) {
        Ok(r) => r,
        Err(e) => {
            return Err(e).context(format!(
                "Kernel build finished for {} but artifacts failed verification",
                t.distro_id
            ));
        }
    };
    eprintln!("[ok] {}: {}", t.distro_id, rel);
    Ok(())
}
