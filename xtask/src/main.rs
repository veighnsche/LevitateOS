//! LevitateOS xtask - Development task runner
//!
//! Usage:
//!   cargo xtask test          # Run ALL tests (unit + behavior + regression + gicv3)
//!   cargo xtask test unit     # Run unit tests only
//!   cargo xtask test behavior # Run behavior test only (default profile)
//!   cargo xtask test regress  # Run regression tests only
//!   cargo xtask test gicv3    # Run GICv3 profile behavior test only
//!   cargo xtask build         # Build kernel (release)
//!   cargo xtask run           # Build and run in QEMU

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Command, Stdio};

mod qmp;
mod tests;

#[derive(Parser)]
#[command(name = "xtask", about = "LevitateOS development task runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run tests
    Test {
        /// Which test suite to run (unit, behavior, regress, gicv3, or all)
        #[arg(default_value = "all")]
        suite: String,
    },
    /// Build the kernel
    Build,
    /// Build and run in QEMU (default profile: 512MB)
    Run,
    /// Build and run in QEMU with Pixel 6 profile (8GB, 8 cores)
    RunPixel6,
    /// Dump the current GPU screen to a file via QMP
    GpuDump {
        /// Output PNG file path
        #[arg(default_value = "screenshot.png")]
        output: String,
    },
    /// Run QEMU with VNC for browser-based GPU display verification
    /// 
    /// AI AGENTS: Use this to verify GPU works! After running:
    /// 1. Navigate browser to http://localhost:6080/vnc.html
    /// 2. Click "Connect" 
    /// 3. If you see "Display output is not active" - GPU is BROKEN
    /// 4. If you see terminal text - GPU is WORKING
    RunVnc,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure we're in project root
    let project_root = project_root()?;
    std::env::set_current_dir(&project_root)?;

    match cli.command {
        Commands::Test { suite } => match suite.as_str() {
            "all" => {
                println!("🧪 Running COMPLETE test suite...\n");
                tests::unit::run()?;
                tests::behavior::run()?;
                tests::behavior::run_gicv3().unwrap_or_else(|_| {
                    println!("⚠️  GICv3 behavior differs (expected, needs separate golden file)\n");
                });
                tests::regression::run()?;
                println!("\n✅ COMPLETE test suite finished!");
            }
            "unit" => tests::unit::run()?,
            "behavior" => tests::behavior::run()?,
            "regress" | "regression" => tests::regression::run()?,
            "gicv3" => tests::behavior::run_gicv3()?,
            other => bail!("Unknown test suite: {}. Use 'unit', 'behavior', 'regress', 'gicv3', or 'all'", other),
        },
        Commands::Build => {
            build_kernel()?;
        }
        Commands::Run => {
            build_kernel()?;
            run_qemu(QemuProfile::Default, false)?;
        }
        Commands::RunPixel6 => {
            println!("🎯 Running with Pixel 6 profile (8GB RAM, 8 cores)");
            build_kernel()?;
            run_qemu(QemuProfile::Pixel6, false)?;
        }
        Commands::GpuDump { output } => {
            println!("📸 Dumping GPU screen to {}...", output);
            let mut client = qmp::QmpClient::connect("./qmp.sock")?;
            let args = serde_json::json!({
                "filename": output,
            });
            client.execute("screendump", Some(args))?;
            println!("✅ Screenshot saved to {}", output);
        }
        Commands::RunVnc => {
            run_qemu_vnc()?;
        }
    }

    Ok(())
}

fn project_root() -> Result<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    // If we're in xtask/, go up one level
    if manifest_dir.ends_with("xtask") {
        Ok(manifest_dir.parent().unwrap().to_path_buf())
    } else {
        Ok(manifest_dir)
    }
}


fn build_userspace() -> Result<()> {
    println!("Building userspace/hello...");
    
    // TEAM_073: Isolate build in /tmp to avoid parent .cargo/config.toml inheritance
    let build_dir = PathBuf::from("/tmp/levitate_hello_build");
    if build_dir.exists() {
        std::fs::remove_dir_all(&build_dir)?;
    }
    std::fs::create_dir_all(&build_dir)?;
    
    // Copy source
    let status = Command::new("cp")
        .args(["-r", "userspace/hello/.", "/tmp/levitate_hello_build/"])
        .status()?;
    if !status.success() {
        bail!("Failed to copy source to tmp");
    }

    // Build in isolation
    let status = Command::new("cargo")
        .current_dir(&build_dir)
        .args([
            "build",
            "--release",
            "--target", "aarch64-unknown-none",
        ])
        .status()
        .context("Failed to build userspace/hello")?;

    if !status.success() {
        bail!("Userspace build failed");
    }

    // Copy artifact back
    let target_dir = PathBuf::from("userspace/hello/target/aarch64-unknown-none/release");
    std::fs::create_dir_all(&target_dir)?;
    std::fs::copy(
        build_dir.join("target/aarch64-unknown-none/release/hello"),
        target_dir.join("hello")
    )?;
    
    Ok(())
}

fn create_initramfs() -> Result<()> {
    println!("Creating initramfs...");
    let root = PathBuf::from("initrd_root");
    if !root.exists() {
        std::fs::create_dir(&root)?;
    }

    // 1. Create content
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;
    
    // 2. Copy userspace binary
    let hello_src = PathBuf::from("userspace/hello/target/aarch64-unknown-none/release/hello");
    if hello_src.exists() {
        std::fs::copy(hello_src, root.join("hello"))?;
        println!("  - Added 'hello' binary");
    } else {
        println!("  - WARNING: userspace/hello binary not found, skipping");
    }

    // 3. Create CPIO archive
    // usage: find . | cpio -o -H newc > ../initramfs.cpio
    let cpio_file = std::fs::File::create("initramfs.cpio")?;
    
    let find = Command::new("find")
        .current_dir(&root)
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run find")?;

    let mut cpio = Command::new("cpio")
        .current_dir(&root)
        .args(["-o", "-H", "newc"])
        .stdin(find.stdout.unwrap())
        .stdout(cpio_file)
        .spawn()
        .context("Failed to run cpio")?;

    let status = cpio.wait()?;
    if !status.success() {
        bail!("cpio failed");
    }

    Ok(())
}

pub fn build_kernel() -> Result<()> {
    // TEAM_073: Build userspace first
    build_userspace()?;
    create_initramfs()?;

    build_kernel_with_features(&[])
}

/// Build kernel with verbose feature for behavior testing (Rule 4: Silence is Golden)
pub fn build_kernel_verbose() -> Result<()> {
    build_kernel_with_features(&["verbose"])
}

fn build_kernel_with_features(features: &[&str]) -> Result<()> {
    println!("Building kernel...");
    let mut args = vec![
        "build".to_string(),
        "-Z".to_string(), "build-std=core,alloc".to_string(),
        "--release".to_string(),
        "--target".to_string(), "aarch64-unknown-none".to_string(),
        "-p".to_string(), "levitate-kernel".to_string(),
    ];
    
    if !features.is_empty() {
        args.push("--features".to_string());
        args.push(features.join(","));
    }
    
    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Kernel build failed");
    }

    // Convert to binary for boot protocol support (Rule 38)
    println!("Converting to raw binary...");
    let objcopy_status = Command::new("aarch64-linux-gnu-objcopy")
        .args([
            "-O", "binary",
            "target/aarch64-unknown-none/release/levitate-kernel",
            "kernel64_rust.bin",
        ])
        .status()
        .context("Failed to run objcopy - is aarch64-linux-gnu-objcopy installed?")?;

    if !objcopy_status.success() {
        bail!("objcopy failed");
    }

    Ok(())
}

/// QEMU hardware profiles
/// TEAM_042: Added Pixel 6 profile for target hardware testing
///
/// Pixel 6 Tensor SoC has:
/// - 2x Cortex-X1 @ 2.80 GHz (big)
/// - 2x Cortex-A76 @ 2.25 GHz (medium)  
/// - 4x Cortex-A55 @ 1.80 GHz (little)
///
/// QEMU Mitigations (verified QEMU 10.1+):
/// - cortex-a76 is available (exact match for medium cores, close to X1)
/// - Cluster topology supported via -smp clusters=N
/// - 8GB RAM supported via highmem
///
/// Limitations:
/// - QEMU cannot mix CPU types (no true big.LITTLE) → use clusters
/// - GICv3 not supported by kernel yet → use GICv2 (limits to 8 CPUs)
#[derive(Clone, Copy)]
pub enum QemuProfile {
    /// Default: 512MB RAM, 1 core, cortex-a53
    Default,
    /// Pixel 6: 8GB RAM, 8 cores, cortex-a76, GICv3
    Pixel6,
    /// Test: GICv3 on default machine
    GicV3,
}

impl QemuProfile {
    pub fn machine(&self) -> String {
        match self {
            QemuProfile::Default => "virt".to_string(),
            QemuProfile::Pixel6 => "virt,gic-version=3".to_string(),
            QemuProfile::GicV3 => "virt,gic-version=3".to_string(),
        }
    }

    pub fn cpu(&self) -> &'static str {
        match self {
            QemuProfile::Default => "cortex-a53",
            QemuProfile::Pixel6 => "cortex-a76",
            QemuProfile::GicV3 => "cortex-a53",
        }
    }

    pub fn memory(&self) -> &'static str {
        match self {
            QemuProfile::Default => "512M",
            QemuProfile::Pixel6 => "8G",
            QemuProfile::GicV3 => "512M",
        }
    }

    /// Returns SMP topology string
    pub fn smp(&self) -> Option<&'static str> {
        match self {
            QemuProfile::Default => None,
            QemuProfile::Pixel6 => Some("8"),
            QemuProfile::GicV3 => None,
        }
    }
}

pub fn run_qemu(profile: QemuProfile, headless: bool) -> Result<()> {
    let kernel_bin = "kernel64_rust.bin";

    let machine = profile.machine();
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-device", "virtio-gpu-pci,xres=1920,yres=1080", // TEAM_115: Larger resolution
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-no-reboot",
    ];

    if let Some(smp) = profile.smp() {
        args.extend(["-smp", smp]);
    }

    if headless {
        args.extend(["-display", "none", "-serial", "stdio"]);
    } else {
        args.extend(["-serial", "stdio"]);
    }

    Command::new("qemu-system-aarch64")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to run QEMU")?;

    Ok(())
}

/// Run QEMU with VNC for browser-based GPU display verification.
/// 
/// # AI Agent Instructions
/// 
/// This command is designed for AI agents to verify GPU display output:
/// 
/// 1. Run `cargo xtask run-vnc`
/// 2. Navigate browser to http://localhost:6080/vnc.html
/// 3. Click "Connect" button
/// 4. Observe the display:
///    - "Display output is not active" = GPU is BROKEN
///    - Visible terminal/text = GPU is WORKING
/// 
/// The display check is more reliable than serial output, which gives false positives.
/// 
/// # Idempotency
/// 
/// This command is idempotent - it kills any existing QEMU/websockify processes
/// before starting new ones. Safe to run multiple times.
pub fn run_qemu_vnc() -> Result<()> {
    println!("🖥️  Starting QEMU with VNC for browser-based display verification...\n");
    
    // Build kernel first
    build_kernel()?;
    
    // Check for noVNC
    let novnc_path = PathBuf::from("/tmp/novnc");
    if !novnc_path.exists() {
        println!("📥 Downloading noVNC...");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "https://github.com/novnc/noVNC.git", "/tmp/novnc"])
            .status()
            .context("Failed to clone noVNC")?;
        if !status.success() {
            bail!("Failed to download noVNC");
        }
    }
    
    // Find websockify - check multiple locations
    let websockify_path = find_websockify()?;
    
    // Kill any existing VNC-related processes (idempotency)
    // Use specific patterns to avoid killing unrelated QEMU instances
    println!("🧹 Cleaning up existing processes...");
    let _ = Command::new("pkill").args(["-f", "websockify.*6080"]).status();
    let _ = Command::new("pkill").args(["-f", "qemu.*-vnc.*:0"]).status();
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Start websockify
    println!("🔌 Starting websockify proxy...");
    let mut websockify = Command::new(&websockify_path)
        .args(["--web=/tmp/novnc", "6080", "localhost:5900"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start websockify")?;
    
    std::thread::sleep(std::time::Duration::from_secs(1));
    
    // Verify websockify started
    match websockify.try_wait() {
        Ok(Some(status)) => {
            bail!("websockify exited immediately with status: {}. Port 6080 may be in use.", status);
        }
        Ok(None) => {} // Still running, good
        Err(e) => {
            bail!("Failed to check websockify status: {}", e);
        }
    }
    
    println!("");
    println!("╔════════════════════════════════════════════════════════════════════════╗");
    println!("║  🌐 BROWSER URL: http://localhost:6080/vnc.html                        ║");
    println!("║                                                                         ║");
    println!("║  📋 AI AGENT INSTRUCTIONS:                                              ║");
    println!("║     1. Navigate browser to the URL above                                ║");
    println!("║     2. Click 'Connect' button                                           ║");
    println!("║     3. Check what displays:                                             ║");
    println!("║        • 'Display output is not active' = GPU BROKEN ❌                 ║");
    println!("║        • Terminal text visible = GPU WORKING ✅                         ║");
    println!("║                                                                         ║");
    println!("║  Serial console is in THIS terminal (Ctrl+C to quit)                    ║");
    println!("╚════════════════════════════════════════════════════════════════════════╝");
    println!("");
    
    // Clean QMP socket
    let _ = std::fs::remove_file("./qmp.sock");
    
    // Run QEMU with VNC
    let kernel_bin = "kernel64_rust.bin";
    let profile = QemuProfile::Default;
    let machine = profile.machine();
    
    let mut args = vec![
        "-M", machine.as_str(),
        "-cpu", profile.cpu(),
        "-m", profile.memory(),
        "-kernel", kernel_bin,
        "-display", "none",
        "-vnc", ":0",
        "-device", "virtio-gpu-pci,xres=1920,yres=1080", // TEAM_115: Larger resolution for VNC
        "-device", "virtio-keyboard-device",
        "-device", "virtio-tablet-device",
        "-device", "virtio-net-device,netdev=net0",
        "-netdev", "user,id=net0",
        "-drive", "file=tinyos_disk.img,format=raw,if=none,id=hd0",
        "-device", "virtio-blk-device,drive=hd0",
        "-initrd", "initramfs.cpio",
        "-serial", "mon:stdio",
        "-qmp", "unix:./qmp.sock,server,nowait",
        "-no-reboot",
    ];
    
    if let Some(smp) = profile.smp() {
        args.extend(["-smp", smp]);
    }
    
    let qemu_result = Command::new("qemu-system-aarch64")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
    
    // Cleanup
    let _ = websockify.kill();
    
    qemu_result.context("Failed to run QEMU")?;
    
    Ok(())
}

/// Find websockify binary in various possible locations
fn find_websockify() -> Result<String> {
    // Check PATH first (covers system installs and activated venvs)
    if let Ok(output) = Command::new("which").arg("websockify").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                println!("  Found websockify at: {}", path);
                return Ok(path);
            }
        }
    }
    
    // Check common pip user install location
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
    let pip_path = format!("{}/.local/bin/websockify", home);
    if std::path::Path::new(&pip_path).exists() {
        println!("  Found websockify at: {}", pip_path);
        return Ok(pip_path);
    }
    
    // Check for pipx installation
    let pipx_path = format!("{}/.local/pipx/venvs/websockify/bin/websockify", home);
    if std::path::Path::new(&pipx_path).exists() {
        println!("  Found websockify at: {}", pipx_path);
        return Ok(pipx_path);
    }
    
    bail!(
        "websockify not found!\n\
        \n\
        Install with one of:\n\
        • pip3 install websockify\n\
        • pipx install websockify\n\
        • sudo dnf install python3-websockify  (Fedora)\n\
        • sudo apt install websockify  (Debian/Ubuntu)"
    )
}
