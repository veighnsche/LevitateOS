use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn boot(n: u8, distro: crate::cli::BootDistro) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let cfg = BootConfig::for_distro(&root, distro);

    match n {
        1 => boot_live_iso(&root, &cfg),
        2 => boot_interactive_checkpoint_2(&root, &cfg),
        4 => boot_installed_disk(&root, &cfg),
        _ => {
            bail!(
                "Checkpoint {n} is automated. Interactive checkpoints: 1 (live), 2 (live tools), 4 (installed)."
            )
        }
    }
}

pub fn test(n: u8, distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--checkpoint", &n.to_string()])
}

pub fn test_up_to(n: u8, distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--up-to", &n.to_string()])
}

pub fn status(distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--status"])
}

pub fn reset(distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--reset"])
}

struct BootConfig {
    iso: PathBuf,
    disk_dir: PathBuf,
    disk_name: &'static str,
    vars_name: &'static str,
    pretty_name: &'static str,
    harness_distro: crate::cli::HarnessDistro,
}

impl BootConfig {
    fn for_distro(root: &Path, distro: crate::cli::BootDistro) -> Self {
        match distro {
            crate::cli::BootDistro::Leviso => Self {
                iso: root.join(".artifacts/out/leviso/levitateos-x86_64.iso"),
                disk_dir: root.join(".artifacts/out/leviso"),
                disk_name: "levitate-test.qcow2",
                vars_name: "levitate-ovmf-vars.fd",
                pretty_name: "LevitateOS",
                harness_distro: crate::cli::HarnessDistro::Levitate,
            },
            crate::cli::BootDistro::Acorn => Self {
                iso: root.join(".artifacts/out/AcornOS/acornos.iso"),
                disk_dir: root.join(".artifacts/out/AcornOS"),
                disk_name: "acorn-test.qcow2",
                vars_name: "acorn-ovmf-vars.fd",
                pretty_name: "AcornOS",
                harness_distro: crate::cli::HarnessDistro::Acorn,
            },
            crate::cli::BootDistro::Iuppiter => Self {
                iso: root.join(".artifacts/out/IuppiterOS/iuppiter-x86_64.iso"),
                disk_dir: root.join(".artifacts/out/IuppiterOS"),
                disk_name: "iuppiter-test.qcow2",
                vars_name: "iuppiter-ovmf-vars.fd",
                pretty_name: "IuppiterOS",
                harness_distro: crate::cli::HarnessDistro::Iuppiter,
            },
        }
    }
}

fn boot_live_iso(root: &Path, cfg: &BootConfig) -> Result<()> {
    if !cfg.iso.is_file() {
        bail!(
            "Missing ISO: {} (build it first, e.g. `just build` for the distro)",
            cfg.iso.display()
        );
    }

    eprintln!("Booting {} live ISO... (Ctrl-A X to exit)", cfg.pretty_name);

    let ovmf = crate::util::repo::ovmf_path(root);

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args([
        "-enable-kvm",
        "-cpu",
        "host",
        "-smp",
        "4",
        "-m",
        "4G",
        "-device",
        "virtio-scsi-pci,id=scsi0",
        "-device",
        "scsi-cd,drive=cdrom0,bus=scsi0.0",
        "-drive",
        &format!(
            "id=cdrom0,if=none,format=raw,readonly=on,file={}",
            cfg.iso.display()
        ),
        "-drive",
        &format!("if=pflash,format=raw,readonly=on,file={}", ovmf.display()),
        "-vga",
        "none",
        "-nographic",
        "-serial",
        "mon:stdio",
        "-no-reboot",
    ]);

    crate::util::tools_env::apply_to_command(&mut cmd, root)?;
    run_checked(&mut cmd)
}

fn boot_interactive_checkpoint_2(root: &Path, cfg: &BootConfig) -> Result<()> {
    eprintln!(
        "Booting {} interactive checkpoint 2 (live tools)... (Ctrl-A X to exit)",
        cfg.pretty_name
    );
    run_install_tests_in_dir(
        root,
        &[
            "--distro",
            cfg.harness_distro.id(),
            "--checkpoint",
            "2",
            "--interactive",
        ],
    )
}

fn boot_installed_disk(root: &Path, cfg: &BootConfig) -> Result<()> {
    let disk = cfg.disk_dir.join(cfg.disk_name);
    let vars = cfg.disk_dir.join(cfg.vars_name);
    let ovmf = crate::util::repo::ovmf_path(root);

    if !disk.is_file() {
        bail!("Missing disk image: {}", disk.display());
    }
    if !vars.is_file() {
        bail!("Missing OVMF vars: {}", vars.display());
    }

    eprintln!(
        "Booting installed {}... (Ctrl-A X to exit)",
        cfg.pretty_name
    );

    let mut cmd = Command::new("qemu-system-x86_64");
    cmd.args([
        "-enable-kvm",
        "-cpu",
        "host",
        "-smp",
        "4",
        "-m",
        "4G",
        "-drive",
        &format!("file={},format=qcow2,if=virtio", disk.display()),
        "-drive",
        &format!("if=pflash,format=raw,readonly=on,file={}", ovmf.display()),
        "-drive",
        &format!("if=pflash,format=raw,file={}", vars.display()),
        "-boot",
        "c",
        "-netdev",
        "user,id=net0",
        "-device",
        "virtio-net-pci,netdev=net0",
        "-vga",
        "none",
        "-nographic",
        "-serial",
        "mon:stdio",
        "-no-reboot",
    ]);

    crate::util::tools_env::apply_to_command(&mut cmd, root)?;
    run_checked(&mut cmd)
}

fn run_install_tests(args: &[&str]) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    run_install_tests_in_dir(&root, args)
}

fn run_install_tests_in_dir(root: &Path, args: &[&str]) -> Result<()> {
    let install_tests_dir = root.join("testing/install-tests");
    if !install_tests_dir.is_dir() {
        bail!(
            "Missing {} (submodule not initialized? try `git submodule update --init --recursive`)",
            install_tests_dir.display()
        );
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&install_tests_dir)
        .args(["run", "--bin", "checkpoints", "--"])
        .args(args);

    crate::util::tools_env::apply_to_command(&mut cmd, root)?;
    run_checked(&mut cmd).with_context(|| {
        format!(
            "Running install-tests checkpoints in {}",
            install_tests_dir.display()
        )
    })
}

fn run_checked(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| "Spawning command")?;
    if !status.success() {
        bail!("Command failed with status {status}");
    }
    Ok(())
}
