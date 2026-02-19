use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn boot(
    n: u8,
    distro: crate::cli::BootDistro,
    inject: Option<String>,
    inject_file: Option<PathBuf>,
    ssh: bool,
    ssh_port: u16,
    ssh_timeout: u64,
    no_shell: bool,
    ssh_private_key: Option<PathBuf>,
) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let cfg = BootConfig::for_distro(&root, distro);

    match n {
        1 => boot_live_iso(
            &root,
            &cfg,
            inject,
            inject_file,
            ssh,
            ssh_port,
            ssh_timeout,
            no_shell,
            ssh_private_key,
        ),
        2 => {
            if ssh {
                bail!(
                    "`--ssh` is only supported for Stage 01; use `cargo xtask stages boot 1 --ssh`."
                );
            }
            boot_interactive_stage_02(&root, &cfg, inject, inject_file)
        }
        4 => {
            if ssh {
                bail!(
                    "`--ssh` is only supported for Stage 01; use `cargo xtask stages boot 1 --ssh`."
                );
            }
            boot_installed_disk(&root, &cfg, inject, inject_file)
        }
        _ => bail!(
            "Stage {n} is automated. Interactive stages: 01 (live), 02 (live tools), 04 (installed)."
        ),
    }
}

pub fn test(
    n: u8,
    distro: crate::cli::HarnessDistro,
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<()> {
    run_install_tests(
        &["--distro", distro.id(), "--stage", &n.to_string()],
        inject,
        inject_file,
    )
}

pub fn test_up_to(
    n: u8,
    distro: crate::cli::HarnessDistro,
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<()> {
    run_install_tests(
        &["--distro", distro.id(), "--up-to", &n.to_string()],
        inject,
        inject_file,
    )
}

pub fn status(distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--status"], None, None)
}

pub fn reset(distro: crate::cli::HarnessDistro) -> Result<()> {
    run_install_tests(&["--distro", distro.id(), "--reset"], None, None)
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
            crate::cli::BootDistro::Levitate => Self {
                iso: root.join(".artifacts/out/levitate/s01-boot/levitateos-x86_64-s01_boot.iso"),
                disk_dir: root.join(".artifacts/out/levitate"),
                disk_name: "levitate-test.qcow2",
                vars_name: "levitate-ovmf-vars.fd",
                pretty_name: "LevitateOS",
                harness_distro: crate::cli::HarnessDistro::Levitate,
            },
            crate::cli::BootDistro::Acorn => Self {
                iso: root.join(".artifacts/out/acorn/s01-boot/acornos-s01_boot.iso"),
                disk_dir: root.join(".artifacts/out/acorn"),
                disk_name: "acorn-test.qcow2",
                vars_name: "acorn-ovmf-vars.fd",
                pretty_name: "AcornOS",
                harness_distro: crate::cli::HarnessDistro::Acorn,
            },
            crate::cli::BootDistro::Iuppiter => Self {
                iso: root.join(".artifacts/out/iuppiter/s01-boot/iuppiter-x86_64-s01_boot.iso"),
                disk_dir: root.join(".artifacts/out/iuppiter"),
                disk_name: "iuppiter-test.qcow2",
                vars_name: "iuppiter-ovmf-vars.fd",
                pretty_name: "IuppiterOS",
                harness_distro: crate::cli::HarnessDistro::Iuppiter,
            },
            crate::cli::BootDistro::Ralph => Self {
                iso: root.join(".artifacts/out/ralph/s01-boot/ralphos-x86_64-s01_boot.iso"),
                disk_dir: root.join(".artifacts/out/ralph"),
                disk_name: "ralph-test.qcow2",
                vars_name: "ralph-ovmf-vars.fd",
                pretty_name: "RalphOS",
                harness_distro: crate::cli::HarnessDistro::Ralph,
            },
        }
    }
}

struct BootInjection {
    path: PathBuf,
    cleanup: bool,
}

impl Drop for BootInjection {
    fn drop(&mut self) {
        if self.cleanup {
            let _ = fs::remove_file(&self.path);
        }
    }
}

fn boot_injection_payload(
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<Option<BootInjection>> {
    if let Some(path) = inject_file {
        if !path.is_file() {
            bail!("--inject-file is not a readable file: {}", path.display());
        }
        return Ok(Some(BootInjection {
            path,
            cleanup: false,
        }));
    }

    let inject = match inject {
        Some(raw) => raw,
        None => return Ok(None),
    };
    let raw = inject.trim();
    if raw.is_empty() {
        return Ok(None);
    }

    let mut lines = Vec::new();
    for entry in raw
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        match entry.split_once('=') {
            Some((key, _value)) if !key.trim().is_empty() => {
                lines.push(entry.to_string());
            }
            _ => {
                bail!(
                    "invalid --inject payload '{}'; expected KEY=VALUE pairs separated by commas",
                    entry
                );
            }
        }
    }
    if lines.is_empty() {
        return Ok(None);
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before UNIX_EPOCH")?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("levitate-boot-injection-{ts}.env"));
    fs::write(&path, format!("{}\n", lines.join("\n")))
        .with_context(|| format!("writing boot injection payload '{}'", path.display()))?;

    Ok(Some(BootInjection {
        path,
        cleanup: true,
    }))
}

fn boot_live_iso(
    root: &Path,
    cfg: &BootConfig,
    inject: Option<String>,
    inject_file: Option<PathBuf>,
    ssh: bool,
    ssh_port: u16,
    ssh_timeout: u64,
    no_shell: bool,
    ssh_private_key: Option<PathBuf>,
) -> Result<()> {
    if !cfg.iso.is_file() {
        bail!(
            "Missing Stage 01 ISO: {} (build it first, e.g. `just build {} 01Boot`)",
            cfg.iso.display(),
            cfg.harness_distro.id()
        );
    }
    let injection = boot_injection_payload(inject, inject_file)?;
    if ssh {
        boot_live_iso_ssh(
            root,
            cfg,
            injection,
            ssh_port,
            ssh_timeout,
            no_shell,
            ssh_private_key,
        )
    } else {
        boot_live_iso_serial(root, cfg, injection, no_shell)
    }
}

fn boot_live_iso_serial(
    root: &Path,
    cfg: &BootConfig,
    injection: Option<BootInjection>,
    no_shell: bool,
) -> Result<()> {
    if no_shell {
        let log_path = temp_log_path("levitate-stage01-serial-boot");
        let mut cmd = qemu_base_command(
            root,
            cfg,
            injection.as_ref().map(|v| v.path.as_path()),
            None,
        )?;
        let child = spawn_qemu_with_log(&mut cmd, &log_path)?;
        monitor_live_iso_serial(child, &log_path)?;
        let _ = fs::remove_file(&log_path);
        return Ok(());
    }

    eprintln!("Booting {} live ISO... (Ctrl-A X to exit)", cfg.pretty_name);
    let mut cmd = qemu_base_command(
        root,
        cfg,
        injection.as_ref().map(|v| v.path.as_path()),
        None,
    )?;
    run_checked(&mut cmd)
}

fn monitor_live_iso_serial(mut child: Child, log_path: &Path) -> Result<()> {
    let booted_message = "switching root to live system";
    let default_timeout = 120u64;
    let timeout_secs = std::env::var("LEVITATE_STAGE01_SERIAL_TIMEOUT")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(default_timeout);
    let deadline = Instant::now() + Duration::from_secs(timeout_secs.max(1));

    loop {
        if let Some(exit_status) = child.try_wait()? {
            let reason = match exit_status.code() {
                Some(code) => format!("QEMU exited with code {code}"),
                None => "QEMU exited by signal".to_string(),
            };
            return bail_with_tail(
                &format!("{reason} before live boot completed"),
                log_path,
                None::<&str>,
            );
        }

        if let Some(pat) = detect_boot_regression(log_path)? {
            let _ = child.kill();
            let _ = child.wait();
            return bail_with_tail(
                &format!("Detected boot regression while waiting for live boot handoff: {pat}"),
                log_path,
                None::<&str>,
            );
        }

        if detect_live_boot_success(log_path, booted_message) {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(());
        }

        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return bail_with_tail(
                &format!("Timed out waiting for Stage 01 serial boot handoff ({timeout_secs}s)"),
                log_path,
                Some("No root-switch handoff marker observed."),
            );
        }

        sleep(Duration::from_secs(1));
    }
}

fn detect_live_boot_success(log_path: &Path, pattern: &str) -> bool {
    let content = match fs::read_to_string(log_path) {
        Ok(raw) => raw,
        Err(_) => return false,
    };

    let needle = pattern.to_lowercase();
    content
        .lines()
        .any(|line| line.to_lowercase().contains(&needle))
}

fn boot_live_iso_ssh(
    root: &Path,
    cfg: &BootConfig,
    injection: Option<BootInjection>,
    ssh_port: u16,
    ssh_timeout: u64,
    no_shell: bool,
    ssh_private_key: Option<PathBuf>,
) -> Result<()> {
    eprintln!(
        "Booting {} live ISO with SSH wait (port 127.0.0.1:{ssh_port})...",
        cfg.pretty_name
    );
    ensure_ssh_port_available(ssh_port)?;

    let mut cmd = qemu_base_command(
        root,
        cfg,
        injection.as_ref().map(|v| v.path.as_path()),
        Some(ssh_port),
    )?;
    let log_path = temp_log_path("levitate-stage01-ssh-boot");
    let child = spawn_qemu_with_log(&mut cmd, &log_path)?;
    let result = monitor_live_iso_ssh(
        child,
        &log_path,
        ssh_port,
        ssh_timeout,
        no_shell,
        ssh_private_key,
    );
    let result = match result {
        Ok(()) => Ok(()),
        Err(err) => {
            let report = maybe_append_log_fault(&log_path);
            if let Some(report) = report {
                bail!("{err}\n{report}");
            }
            bail!("{:#}", err);
        }
    };
    let _ = fs::remove_file(&log_path);
    result
}

fn monitor_live_iso_ssh(
    mut child: Child,
    log_path: &Path,
    ssh_port: u16,
    ssh_timeout: u64,
    no_shell: bool,
    ssh_private_key: Option<PathBuf>,
) -> Result<()> {
    let known_hosts = temp_file_path("levitate-stage01-ssh-known-hosts");
    fs::write(&known_hosts, "").context("creating known-hosts scratch file")?;
    let deadline = Instant::now() + Duration::from_secs(ssh_timeout.max(1));
    let key = resolve_ssh_private_key(ssh_private_key)?;
    let mut hook_seen = false;
    let mut lines_seen = 0usize;

    loop {
        let _ = emit_new_log_lines(log_path, &mut lines_seen)?;

        if let Some(exit_status) = child.try_wait()? {
            let reason = match exit_status.code() {
                Some(code) => format!("QEMU exited with code {code}"),
                None => "QEMU exited by signal".to_string(),
            };
            let _ = fs::remove_file(&known_hosts);
            return bail_with_tail(
                &format!("{reason} before SSH became ready"),
                log_path,
                None::<&str>,
            );
        }

        if let Some(pat) = detect_boot_regression(log_path)? {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&known_hosts);
            return bail_with_tail(
                &format!(
                    "Detected boot regression while waiting for SSH (sshd failure or locale warning): {pat}"
                ),
                log_path,
                None::<&str>,
            );
        }

        if !hook_seen {
            if let Some(pat) = detect_stage01_boot_hook(log_path)? {
                hook_seen = true;
                eprintln!(
                    "Boot hook observed ({pat}); waiting for SSH readiness on 127.0.0.1:{ssh_port}..."
                );
            }
        } else if can_ssh_connect(ssh_port, &key, &known_hosts)? {
            if no_shell {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&known_hosts);
                return Ok(());
            }

            let status = run_interactive_ssh(ssh_port, &key, &known_hosts, &mut child);
            let _ = fs::remove_file(&known_hosts);
            return status;
        }

        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            let _ = fs::remove_file(&known_hosts);
            let mut extra = format!("No successful SSH handshake observed.");
            if !hook_seen {
                extra = format!("No boot hook observed yet after {ssh_timeout}s.");
            }
            return bail_with_tail(
                &format!("Timed out waiting for SSH readiness ({ssh_timeout}s)"),
                log_path,
                Some(&extra),
            );
        }
        sleep(Duration::from_secs(1));
    }
}

fn detect_stage01_boot_hook(log_path: &Path) -> Result<Option<String>> {
    let raw = match fs::read_to_string(log_path) {
        Ok(raw) => raw,
        Err(_) => return Ok(None),
    };

    for pat in ["___SHELL_READY___", "___PROMPT___"] {
        if raw.contains(pat) {
            return Ok(Some(pat.to_string()));
        }
    }
    Ok(None)
}

fn emit_new_log_lines(log_path: &Path, line_cursor: &mut usize) -> Result<()> {
    let mut lines = match fs::read_to_string(log_path) {
        Ok(raw) => raw.lines().map(str::to_string).collect::<Vec<_>>(),
        Err(_) => return Ok(()),
    };

    let total_lines = lines.len();
    if total_lines <= *line_cursor {
        return Ok(());
    }

    for line in lines.drain(..*line_cursor) {
        let _ = line;
    }
    for line in lines {
        println!("{}", strip_ansi_escapes(&line));
    }
    *line_cursor = total_lines;
    Ok(())
}

fn strip_ansi_escapes(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;

    while i < bytes.len() {
        if bytes[i] != b'\x1b' {
            out.push(bytes[i] as char);
            i += 1;
            continue;
        }

        if i + 1 >= bytes.len() {
            break;
        }

        match bytes[i + 1] {
            b'[' => {
                i += 2;
                while i < bytes.len() {
                    let b = bytes[i];
                    if (0x40..=0x7e).contains(&b) {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            b']' => {
                i += 2;
                while i < bytes.len() {
                    if bytes[i] == 0x07 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == b'\x1b' && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            _ => {
                i += 2;
            }
        }
    }

    out
}

fn run_interactive_ssh(
    ssh_port: u16,
    private_key: &Path,
    known_hosts: &Path,
    qemu: &mut Child,
) -> Result<()> {
    let mut args = common_ssh_args(private_key, ssh_port, known_hosts);
    args.push("-o".to_string());
    args.push("BatchMode=no".to_string());
    args.push("root@127.0.0.1".to_string());
    let status = Command::new("ssh")
        .args(&args)
        .status()
        .context("launching interactive SSH session")?;

    let _ = qemu.kill();
    let _ = qemu.wait();
    if status.success() {
        Ok(())
    } else {
        bail!("interactive SSH session exited with status {status}")
    }
}

fn can_ssh_connect(ssh_port: u16, private_key: &Path, known_hosts: &Path) -> Result<bool> {
    let mut args = common_ssh_args(private_key, ssh_port, known_hosts);
    args.push("root@127.0.0.1".to_string());
    args.push("true".to_string());
    let status = Command::new("ssh")
        .args(&args)
        .status()
        .context("checking SSH readiness")?;
    Ok(status.success())
}

fn resolve_ssh_private_key(arg: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = arg {
        if !path.is_file() {
            bail!("SSH private key does not exist: {}", path.display());
        }
        return Ok(path);
    }

    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME is not set; pass --ssh-private-key")?;
    let fallback = home.join(".ssh").join("id_ed25519");
    if !fallback.is_file() {
        bail!(
            "--ssh-private-key was not provided and {} does not exist",
            fallback.display()
        );
    }
    Ok(fallback)
}

fn common_ssh_args(key: &Path, ssh_port: u16, known_hosts: &Path) -> Vec<String> {
    vec![
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=2".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-o".to_string(),
        format!("UserKnownHostsFile={}", known_hosts.display()),
        "-o".to_string(),
        "IdentitiesOnly=yes".to_string(),
        "-i".to_string(),
        key.display().to_string(),
        "-p".to_string(),
        ssh_port.to_string(),
    ]
}

fn boot_interactive_stage_02(
    root: &Path,
    cfg: &BootConfig,
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<()> {
    eprintln!(
        "Booting {} interactive Stage 02 (live tools)... (Ctrl-A X to exit)",
        cfg.pretty_name
    );
    run_install_tests_in_dir(
        root,
        &[
            "--distro",
            cfg.harness_distro.id(),
            "--stage",
            "2",
            "--interactive",
        ],
        inject,
        inject_file,
    )
}

fn boot_installed_disk(
    root: &Path,
    cfg: &BootConfig,
    _inject: Option<String>,
    _inject_file: Option<PathBuf>,
) -> Result<()> {
    let disk = cfg.disk_dir.join(cfg.disk_name);
    let vars = cfg.disk_dir.join(cfg.vars_name);
    let ovmf = crate::util::repo::ovmf_path(root)?;

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

fn run_install_tests(
    args: &[&str],
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    run_install_tests_in_dir(&root, args, inject, inject_file)
}

fn run_install_tests_in_dir(
    root: &Path,
    args: &[&str],
    inject: Option<String>,
    inject_file: Option<PathBuf>,
) -> Result<()> {
    let install_tests_dir = root.join("testing/install-tests");
    if !install_tests_dir.is_dir() {
        bail!(
            "Missing {} (submodule not initialized? try `git submodule update --init --recursive`)",
            install_tests_dir.display()
        );
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&install_tests_dir)
        .args(["run", "--bin", "stages", "--"])
        .args(args);
    if let Some(path) = &inject_file {
        let path = path.to_string_lossy();
        cmd.args(["--inject-file", path.as_ref()]);
    }
    if let Some(payload) = inject {
        cmd.args(["--inject", &payload]);
    }

    crate::util::tools_env::apply_to_command(&mut cmd, root)?;
    run_checked(&mut cmd).with_context(|| {
        format!(
            "Running install-tests stages in {}",
            install_tests_dir.display()
        )
    })
}

fn qemu_base_command(
    root: &Path,
    cfg: &BootConfig,
    injection: Option<&Path>,
    ssh_port: Option<u16>,
) -> Result<Command> {
    let ovmf = crate::util::repo::ovmf_path(root)?;
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
    if let Some(injection) = injection {
        let fw_cfg = format!(
            "name=opt/levitate/boot-injection,file={}",
            injection.display()
        );
        cmd.args(["-fw_cfg", &fw_cfg]);
    }
    if let Some(ssh_port) = ssh_port {
        cmd.args([
            "-netdev",
            &format!("user,id=net0,hostfwd=tcp:127.0.0.1:{ssh_port}-:22"),
            "-device",
            "virtio-net-pci,netdev=net0",
        ]);
    } else {
        cmd.args(["-netdev", "user,id=net0"]);
        cmd.args(["-device", "virtio-net-pci,netdev=net0"]);
    }

    crate::util::tools_env::apply_to_command(&mut cmd, root)?;
    Ok(cmd)
}

fn spawn_qemu_with_log(cmd: &mut Command, log_path: &Path) -> Result<Child> {
    let log_out = File::create(log_path)
        .with_context(|| format!("creating QEMU log file '{}'", log_path.display()))?;
    let log_err = log_out
        .try_clone()
        .with_context(|| format!("duplicating QEMU log file '{}'", log_path.display()))?;
    cmd.stdout(Stdio::from(log_out));
    cmd.stderr(Stdio::from(log_err));
    cmd.stdin(Stdio::null());
    let child = cmd.spawn().context("Spawning QEMU for SSH boot")?;
    Ok(child)
}

fn detect_boot_regression(log_path: &Path) -> Result<Option<String>> {
    if !log_path.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(log_path).unwrap_or_default();
    if content.is_empty() {
        return Ok(None);
    }

    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("could not set up host forwarding rule") {
            return Ok(Some(format!("hostfwd setup failed: {line}")));
        }
        if lower.contains("warning") && lower.contains("locale") {
            return Ok(Some(format!("locale warning: {line}")));
        }
        if lower.contains("failed to start sshd.service")
            || lower.contains("sshd.service: failed with result")
            || lower.contains("start request repeated too quickly")
            || lower.contains("failed to start ssh.service")
        {
            return Ok(Some(format!("sshd failure: {line}")));
        }
    }

    Ok(None)
}

fn ensure_ssh_port_available(ssh_port: u16) -> Result<()> {
    match TcpListener::bind(("127.0.0.1", ssh_port)) {
        Ok(listener) => {
            drop(listener);
            Ok(())
        }
        Err(err) => bail!(
            "local SSH host port {ssh_port} is unavailable (bind error: {err}). \
            Use `--ssh-port` to choose a free port."
        ),
    }
}

fn temp_log_path(prefix: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{ts}.log"))
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|dur| dur.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}-{ts}"))
}

fn dump_log_tail(log_path: &Path, lines: usize) -> String {
    match fs::read_to_string(log_path) {
        Ok(raw) => {
            let mut output = Vec::new();
            for line in raw
                .lines()
                .rev()
                .take(lines)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
            {
                output.push(line);
            }
            output.join("\n")
        }
        Err(_) => String::new(),
    }
}

fn maybe_append_log_fault(log_path: &Path) -> Option<String> {
    let tail = dump_log_tail(log_path, 120);
    if tail.is_empty() {
        None
    } else {
        Some(format!("Last log lines:\n{tail}"))
    }
}

fn bail_with_tail(message: &str, log_path: &Path, extra: Option<&str>) -> Result<()> {
    let tail = dump_log_tail(log_path, 120);
    let detail = if tail.is_empty() {
        String::new()
    } else {
        format!("\nLast log lines:\n{tail}")
    };
    let tail_extra = extra.unwrap_or("");
    if !tail_extra.is_empty() {
        bail!("{message}\n{tail_extra}{detail}");
    }
    bail!("{message}{detail}");
}

fn run_checked(cmd: &mut Command) -> Result<()> {
    let status = cmd.status().with_context(|| "Spawning command")?;
    if !status.success() {
        bail!("Command failed with status {status}");
    }
    Ok(())
}
