use anyhow::{Context, Result};

pub fn run(
    distro: crate::cli::Distro,
    rebuild_even_if_verified: bool,
    autofix: super::common::AutoFixOptions,
) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let t = super::common::target_for(distro);

    eprintln!("[info] Repo: {}", root.display());
    eprintln!(
        "[info] Target: {} ({}{})",
        t.distro_id, t.kernel.version, t.kernel.localversion
    );

    let need_build = rebuild_even_if_verified || !super::common::kernel_is_built(&root, &t);
    if need_build {
        super::common::enforce_build_hours()?;
    }

    if !rebuild_even_if_verified && super::common::kernel_is_built(&root, &t) {
        eprintln!("[skip] {} kernel already built+verified", t.distro_id);
        return Ok(());
    }

    let recipe_bin = super::common::build_recipe_bin(&root).context("Building recipe binary")?;

    eprintln!("[step] Build kernel: {}", t.distro_id);
    if let Err(e) = super::common::build_kernel_via_recipe(
        &recipe_bin,
        &root,
        t.distro_id,
        rebuild_even_if_verified,
        t.kernel,
        t.module_install_path,
        &autofix,
    ) {
        return Err(e).context(format!("Kernel build failed for {}", t.distro_id));
    }

    let rel = match super::common::verify_one(&root, &t) {
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
