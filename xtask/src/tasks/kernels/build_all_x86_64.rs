use anyhow::{Context, Result, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run(rebuild_even_if_verified: bool) -> Result<()> {
    let root = crate::util::repo::repo_root()?;

    eprintln!("[info] Repo: {}", root.display());

    ensure_output_link(&root, "leviso")?;
    ensure_output_link(&root, "AcornOS")?;
    ensure_output_link(&root, "IuppiterOS")?;
    ensure_output_link(&root, "RalphOS")?;

    let (lts_version, lts_sha256) = parse_levitate_kernel_spec(&root)
        .context("Failed to parse LEVITATE_KERNEL from distro-spec")?;
    eprintln!("[info] LTS kernel (from distro-spec): {}", lts_version);

    maybe_build(&root, rebuild_even_if_verified, "AcornOS", "-acorn", || {
        run_bash(&format!(
            "cd \"{}\" && LEVITATE_DISABLE_KERNEL_THEFT=1 cargo run -- build --dangerously-waste-the-users-time kernel",
            root.join("AcornOS").display()
        ))
    })?;

    maybe_build(
        &root,
        rebuild_even_if_verified,
        "IuppiterOS",
        "-iuppiter",
        || {
            run_bash(&format!(
                "cd \"{}\" && LEVITATE_DISABLE_KERNEL_THEFT=1 cargo run -- build --dangerously-waste-the-users-time kernel",
                root.join("IuppiterOS").display()
            ))
        },
    )?;

    maybe_build(&root, rebuild_even_if_verified, "RalphOS", "-ralph", || {
        build_ralph_kernel(&root, &lts_version, &lts_sha256)
    })?;

    Ok(())
}

fn maybe_build<F>(
    root: &Path,
    rebuild_even_if_verified: bool,
    distro_dir: &str,
    want_suffix: &str,
    build: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    if !rebuild_even_if_verified && kernel_is_built(root, distro_dir, want_suffix) {
        eprintln!("[skip] {distro_dir} kernel already built+verified");
        return Ok(());
    }

    enforce_build_hours()?;

    // Purge partial/stale kernel payload first so we don't end up with mismatched artifacts.
    let out_dir = root.join(".artifacts/out").join(distro_dir);
    rm_rf(out_dir.join("kernel-build"))?;
    rm_rf(out_dir.join("staging/lib/modules"))?;
    rm_rf(out_dir.join("staging/usr/lib/modules"))?;
    rm_file(out_dir.join("staging/boot/vmlinuz"))?;

    eprintln!("[step] Build kernel: {distro_dir}");
    build()
}

fn kernel_is_built(root: &Path, distro_dir: &str, want_suffix: &str) -> bool {
    verify_one(root, distro_dir, want_suffix).is_ok()
}

fn verify_one(root: &Path, distro_dir: &str, want_suffix: &str) -> Result<String> {
    let out_dir = root.join(".artifacts/out").join(distro_dir);
    let rel_file = out_dir.join("kernel-build/include/config/kernel.release");
    let vmlinuz = out_dir.join("staging/boot/vmlinuz");

    if !rel_file.is_file() {
        bail!("Missing kernel.release: {}", rel_file.display());
    }
    if !vmlinuz.is_file() {
        bail!("Missing vmlinuz: {}", vmlinuz.display());
    }

    let rel =
        fs::read_to_string(&rel_file).with_context(|| format!("Reading {}", rel_file.display()))?;
    let rel = rel.trim_end_matches(['\n', '\r']).to_string();

    if !rel.ends_with(want_suffix) {
        bail!(
            "{distro_dir} kernel.release '{rel}' does not end with '{want_suffix}' (theft mode?)"
        );
    }

    let m1 = out_dir.join(format!("staging/lib/modules/{rel}"));
    let m2 = out_dir.join(format!("staging/usr/lib/modules/{rel}"));
    if !m1.is_dir() && !m2.is_dir() {
        bail!(
            "Missing modules dir for {distro_dir} ({rel}) under staging/{{lib,usr/lib}}/modules/"
        );
    }

    Ok(rel)
}

fn enforce_build_hours() -> Result<()> {
    let hhmm = run_capture("date", &["+%H%M"])?;
    let hhmm = hhmm.trim();
    let now = run_capture("date", &["+%Y-%m-%d %H:%M:%S %z"])?;
    let now = now.trim();

    let hhmm_num: i32 = hhmm
        .parse()
        .with_context(|| format!("Parsing date +%H%M output: '{hhmm}'"))?;

    // Allowed: 23:00-23:59, 00:00-10:00 (inclusive)
    if hhmm_num >= 2300 || hhmm_num <= 1000 {
        return Ok(());
    }

    eprintln!(
        "[policy] Refusing to build kernels outside the allowed window.\n\
         Allowed local time window: 23:00 (11pm) through 10:00 (10am).\n\
         Current local time: {now}\n\
\n\
         If you really intend to build right now, rerun later during the window."
    );
    std::process::exit(3);
}

fn ensure_output_link(root: &Path, distro_dir: &str) -> Result<()> {
    let link_path = root.join(distro_dir).join("output");
    let target_dir = root.join(".artifacts/out").join(distro_dir);
    fs::create_dir_all(&target_dir)?;

    if link_path.is_symlink() {
        return Ok(());
    }
    if link_path.exists() {
        bail!(
            "{} exists but is not a symlink. Refusing to replace it. Move it out of the way, or migrate it into .artifacts/out/{distro_dir} first.",
            link_path.display()
        );
    }

    #[cfg(unix)]
    {
        let target_path = PathBuf::from("../.artifacts/out").join(distro_dir);
        std::os::unix::fs::symlink(&target_path, &link_path).with_context(|| {
            format!(
                "Creating symlink {} -> {}",
                link_path.display(),
                target_path.display()
            )
        })?;
    }

    #[cfg(not(unix))]
    {
        bail!("ensure_output_link is only implemented on unix platforms");
    }

    Ok(())
}

fn parse_levitate_kernel_spec(root: &Path) -> Result<(String, String)> {
    let kernel_spec = root.join("distro-spec/src/shared/kernel.rs");
    let s = fs::read_to_string(&kernel_spec)
        .with_context(|| format!("Reading {}", kernel_spec.display()))?;

    // We mimic the script intent: find `pub const LEVITATE_KERNEL` block, then `version:` and `sha256:`.
    let mut in_block = false;
    let mut version: Option<String> = None;
    let mut sha256: Option<String> = None;

    for line in s.lines() {
        if !in_block && line.contains("pub const LEVITATE_KERNEL") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }

        if version.is_none() && line.contains("version:") {
            version = extract_rust_string_literal(line);
        }
        if sha256.is_none() && line.contains("sha256:") {
            sha256 = extract_rust_string_literal(line);
        }

        if line.contains("};") && version.is_some() && sha256.is_some() {
            break;
        }
    }

    let version = version.context("Missing version in LEVITATE_KERNEL block")?;
    let sha256 = sha256.context("Missing sha256 in LEVITATE_KERNEL block")?;
    Ok((version, sha256))
}

fn extract_rust_string_literal(line: &str) -> Option<String> {
    let start = line.find('"')?;
    let rest = &line[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn build_ralph_kernel(root: &Path, lts_version: &str, lts_sha256: &str) -> Result<()> {
    run_cmd(
        Command::new("cargo")
            .current_dir(root)
            .args(["build", "-p", "levitate-recipe"]),
    )?;

    let recipe_bin = root.join("target/debug/recipe");
    if !recipe_bin.is_file() {
        bail!("Expected recipe binary at {}", recipe_bin.display());
    }

    let recipe_rhai = root.join("distro-builder/recipes/linux-base.rhai");
    let build_dir = root.join("RalphOS/downloads");
    let recipes_path = root.join("distro-builder/recipes");

    let mut cmd = Command::new(&recipe_bin);
    cmd.current_dir(root)
        .arg("install")
        .arg(recipe_rhai)
        .args(["--build-dir", build_dir.to_string_lossy().as_ref()])
        .args(["--recipes-path", recipes_path.to_string_lossy().as_ref()])
        .args(["--define", &format!("KERNEL_VERSION={lts_version}")])
        .args(["--define", &format!("KERNEL_SHA256={lts_sha256}")])
        .args(["--define", "KERNEL_LOCALVERSION=-ralph"]);

    run_cmd(&mut cmd)
}

fn run_bash(script: &str) -> Result<()> {
    run_cmd(Command::new("bash").args(["-lc", script]))
}

fn run_capture(prog: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(prog)
        .args(args)
        .output()
        .with_context(|| format!("Running {prog}"))?;
    if !out.status.success() {
        bail!("{prog} failed with status {}", out.status);
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn run_cmd(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| "Spawning command")?;
    if !status.success() {
        bail!("Command failed with status {status}");
    }
    Ok(())
}

fn rm_rf(path: PathBuf) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(&path).with_context(|| format!("Removing {}", path.display()))?;
    } else {
        rm_file(path)?;
    }
    Ok(())
}

fn rm_file(path: PathBuf) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    fs::remove_file(&path).with_context(|| format!("Removing {}", path.display()))?;
    Ok(())
}
