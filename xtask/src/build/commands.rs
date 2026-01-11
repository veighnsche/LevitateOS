use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Stdio};
use crate::disk;

#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (Kernel + Userspace + Disk + Eyra coreutils)
    All,
    /// Build kernel only
    Kernel,
    /// Build userspace only
    Userspace,
    /// Build initramfs only
    Initramfs,
    /// Build bootable Limine ISO (includes Eyra coreutils)
    Iso,
    /// Build Eyra-based userspace utilities (uutils coreutils)
    /// TEAM_367: Auto-discovers and builds all utilities in crates/userspace/eyra/
    Eyra {
        /// Target architecture (x86_64 or aarch64)
        #[arg(long, default_value = "x86_64")]
        arch: String,
        /// Build only a specific utility
        #[arg(long)]
        only: Option<String>,
    },
}

// TEAM_369: Eyra is always enabled (provides std support)
pub fn build_all(arch: &str) -> Result<()> {
    // TEAM_369: Always build Eyra utilities (provides std)
    println!("🔧 Building Eyra utilities...");
    build_eyra(arch, None)?;
    
    // TEAM_073: Build userspace first
    build_userspace(arch)?;
    create_initramfs(arch)?;
    // TEAM_121: Ensure disk image is populated
    disk::install_userspace_to_disk(arch)?;

    build_kernel_with_features(&[], arch)
}

pub fn build_kernel_only(arch: &str) -> Result<()> {
    build_kernel_with_features(&[], arch)
}

/// Build kernel with verbose feature for behavior testing (Rule 4: Silence is Golden)
pub fn build_kernel_verbose(arch: &str) -> Result<()> {
    build_kernel_with_features(&["verbose"], arch)
}

pub fn build_userspace(arch: &str) -> Result<()> {
    println!("Building userspace workspace for {}...", arch);
    
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // TEAM_120: Build the entire userspace workspace
    // We build in-place now as the workspace isolation issues should be resolved
    // by individual build.rs scripts and correct linker arguments.
    let status = Command::new("cargo")
        .current_dir("crates/userspace")
        .args([
            "build",
            "--release",
            "--workspace",
            "--target", target,
        ])
        .status()
        .context("Failed to build userspace workspace")?;

    if !status.success() {
        bail!("Userspace workspace build failed");
    }

    Ok(())
}

// TEAM_369: Always includes Eyra binaries (provides std support)
pub fn create_initramfs(arch: &str) -> Result<()> {
    println!("Creating initramfs for {}...", arch);
    let root = PathBuf::from("initrd_root");
    
    // TEAM_292: Always clean initrd_root to ensure correct arch binaries
    // Without this, stale binaries from other architectures persist
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    // 1. Create content
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;
    
    // 2. Copy userspace binaries
    let binaries = crate::get_binaries(arch)?;
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };
    print!("📦 Creating initramfs ({} binaries)... ", binaries.len());
    let mut count = 0;
    for bin in &binaries {
        let src = PathBuf::from(format!("crates/userspace/target/{}/release/{}", target, bin));
        if src.exists() {
            std::fs::copy(&src, root.join(bin))?;
            count += 1;
        }
    }
    
    // TEAM_380: Add eyra-hello if it exists (static-pie test binary)
    let eyra_target = match arch {
        "aarch64" => "aarch64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        _ => "",
    };
    // eyra-hello is now in the eyra workspace target directory
    let eyra_src = PathBuf::from(format!("crates/userspace/eyra/target/{}/release/eyra-hello", eyra_target));
    if eyra_src.exists() {
        std::fs::copy(&eyra_src, root.join("eyra-hello"))?;
        count += 1;
    }
    
    // TEAM_404: Add brush shell (Eyra-based shell with std support)
    let brush_src = PathBuf::from(format!("crates/userspace/eyra/target/{}/release/brush", eyra_target));
    if brush_src.exists() {
        std::fs::copy(&brush_src, root.join("brush"))?;
        count += 1;
        println!("  📦 Added brush shell");
    }

    // TEAM_430: Add c-gull syscall test
    let cgull_test_src = PathBuf::from(format!("crates/userspace/eyra/target/{}/release/cgull-test", eyra_target));
    if cgull_test_src.exists() {
        std::fs::copy(&cgull_test_src, root.join("cgull-test"))?;
        count += 1;
        println!("  📦 Added cgull-test");
    }

    // TEAM_424: Add syscall conformance test
    let conformance_src = PathBuf::from(format!("crates/userspace/eyra/target/{}/release/syscall-conformance", eyra_target));
    if conformance_src.exists() {
        std::fs::copy(&conformance_src, root.join("syscall-conformance"))?;
        count += 1;
        println!("  📦 Added syscall-conformance");
    }

    // c-gull libc test binary (built with standalone c-gull sysroot)
    let hello_cgull_src = PathBuf::from(format!("toolchain/libc-levitateos/test/rust-test/target/{}/release/hello_rust", eyra_target));
    if hello_cgull_src.exists() {
        std::fs::copy(&hello_cgull_src, root.join("hello-cgull"))?;
        count += 1;
        println!("  📦 Added hello-cgull (c-gull libc test)");
    }

    // TEAM_380: Copy coreutils multi-call binary and create symlinks
    let coreutils_src = PathBuf::from(format!(
        "crates/userspace/eyra/coreutils/target/{}/release/coreutils",
        eyra_target
    ));
    
    // Full pure-Rust utility list (excluding expr which needs onig C library)
    let eyra_utils = [
        // Basic file operations
        "cat", "cp", "dd", "ln", "ls", "mkdir", "mv", "rm", "rmdir", "touch",
        // Text processing (expr excluded - needs onig)
        "basename", "comm", "csplit", "cut", "dirname", "echo", "expand", 
        "fmt", "fold", "head", "join", "nl", "numfmt", "od", "paste",
        "pr", "printenv", "printf", "ptx", "seq", "shuf", "sort", "split",
        "sum", "tac", "tail", "tr", "truncate", "tsort", "unexpand", "uniq", "wc",
        // File utilities
        "dir", "dircolors", "df", "du", "link", "mktemp", "more", "readlink",
        "realpath", "shred", "sleep", "tee", "test", "unlink", "vdir", "yes",
        // Encoding
        "base32", "base64", "basenc",
        // Checksums
        "cksum", "hashsum",
        // Misc
        "env", "factor", "false", "pwd", "true",
    ];
    
    if coreutils_src.exists() {
        // Copy the multi-call binary
        std::fs::copy(&coreutils_src, root.join("coreutils"))?;
        count += 1;
        
        // Create symlinks for each utility
        for util in &eyra_utils {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let link_path = root.join(util);
                // Remove existing file/link if present
                let _ = std::fs::remove_file(&link_path);
                symlink("coreutils", &link_path)?;
            }
            #[cfg(not(unix))]
            {
                // On non-unix, copy the binary instead
                std::fs::copy(&coreutils_src, root.join(util))?;
            }
        }
        println!("  📦 Added coreutils + {} symlinks", eyra_utils.len());
        // Don't count symlinks as separate binaries for the total
    } else {
        println!("  ⚠️  Coreutils not found at {}", coreutils_src.display());
    }
    
    println!("[DONE] ({} added)", count);

    // 3. Create CPIO archive
    // TEAM_327: Use arch-specific filename to prevent cross-arch contamination
    // usage: find . | cpio -o -H newc > ../initramfs_{arch}.cpio
    let cpio_filename = format!("initramfs_{}.cpio", arch);
    let cpio_file = std::fs::File::create(&cpio_filename)?;
    
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

/// TEAM_374: Create test-specific initramfs with Eyra utilities and test runner.
/// Includes init, shell, and eyra-test-runner for comprehensive testing.
pub fn create_test_initramfs(arch: &str) -> Result<()> {
    println!("Creating test initramfs for {}...", arch);
    let root = PathBuf::from("initrd_test_root");
    
    // Clean and create directory
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    let eyra_target = match arch {
        "aarch64" => "aarch64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };
    
    let bare_target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // TEAM_374: Copy init and shell for boot
    let init_src = PathBuf::from(format!("crates/userspace/target/{}/release/init", bare_target));
    let shell_src = PathBuf::from(format!("crates/userspace/target/{}/release/shell", bare_target));
    
    if init_src.exists() {
        std::fs::copy(&init_src, root.join("init"))?;
    }
    if shell_src.exists() {
        std::fs::copy(&shell_src, root.join("shell"))?;
    }
    
    // Create hello.txt for cat test
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;

    // TEAM_380: Copy coreutils multi-call binary and create symlinks
    let coreutils_src = PathBuf::from(format!(
        "crates/userspace/eyra/coreutils/target/{}/release/coreutils",
        eyra_target
    ));
    
    // Full pure-Rust utility list (excluding expr which needs onig C library)
    let eyra_utils = [
        // Basic file operations
        "cat", "cp", "dd", "ln", "ls", "mkdir", "mv", "rm", "rmdir", "touch",
        // Text processing (expr excluded - needs onig)
        "basename", "comm", "csplit", "cut", "dirname", "echo", "expand", 
        "fmt", "fold", "head", "join", "nl", "numfmt", "od", "paste",
        "pr", "printenv", "printf", "ptx", "seq", "shuf", "sort", "split",
        "sum", "tac", "tail", "tr", "truncate", "tsort", "unexpand", "uniq", "wc",
        // File utilities
        "dir", "dircolors", "df", "du", "link", "mktemp", "more", "readlink",
        "realpath", "shred", "sleep", "tee", "test", "unlink", "vdir", "yes",
        // Encoding
        "base32", "base64", "basenc",
        // Checksums
        "cksum", "hashsum",
        // Misc
        "env", "factor", "false", "pwd", "true",
    ];
    let mut count = 0;
    
    if coreutils_src.exists() {
        // Copy the multi-call binary
        std::fs::copy(&coreutils_src, root.join("coreutils"))?;
        count += 1;
        
        // Create symlinks for each utility
        for util in &eyra_utils {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let link_path = root.join(util);
                let _ = std::fs::remove_file(&link_path);
                symlink("coreutils", &link_path)?;
            }
            #[cfg(not(unix))]
            {
                std::fs::copy(&coreutils_src, root.join(util))?;
            }
        }
    } else {
        bail!("Coreutils not found. Run 'cargo xtask build eyra' first.");
    }
    
    // Copy eyra-test-runner from the eyra workspace
    let eyra_target_dir = PathBuf::from(format!("crates/userspace/eyra/target/{}/release", eyra_target));
    let test_runner_src = eyra_target_dir.join("eyra-test-runner");
    if test_runner_src.exists() {
        std::fs::copy(&test_runner_src, root.join("eyra-test-runner"))?;
        count += 1;
    }
    
    println!("📦 Test initramfs: coreutils + {} symlinks + init/shell", eyra_utils.len());

    // Create CPIO archive
    let cpio_file = std::fs::File::create("initramfs_test.cpio")?;
    
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

    println!("✅ Created initramfs_test.cpio");
    Ok(())
}

fn build_kernel_with_features(features: &[&str], arch: &str) -> Result<()> {
    println!("Building kernel for {}...", arch);
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let mut args = vec![
        "build".to_string(),
        "-Z".to_string(), "build-std=core,alloc".to_string(),
        "--release".to_string(),
        "--target".to_string(), target.to_string(),
        "-p".to_string(), "levitate-kernel".to_string(),  // TEAM_426: Only build kernel, not all workspace members
    ];

    if !features.is_empty() {
        args.push("--features".to_string());
        args.push(features.join(","));
    }

    // Kernel is its own workspace - build from kernel directory
    let status = Command::new("cargo")
        .current_dir("crates/kernel")
        .args(&args)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Kernel build failed");
    }

    // Convert to binary for boot protocol support (Rule 38)
    if arch == "aarch64" {
        println!("Converting to raw binary...");
        let objcopy_status = Command::new("aarch64-linux-gnu-objcopy")
            .args([
                "-O", "binary",
                "crates/kernel/target/aarch64-unknown-none/release/levitate-kernel",
                "kernel64_rust.bin",
            ])
            .status()
            .context("Failed to run objcopy - is aarch64-linux-gnu-objcopy installed?")?;

        if !objcopy_status.success() {
            bail!("objcopy failed");
        }
    } else {
        // x86_64 uses multiboot2 (ELF) directly or needs different conversion
        println!("x86_64 kernel build complete (ELF format for multiboot2)");
    }

    Ok(())
}

/// TEAM_283: Build a bootable Limine ISO
// TEAM_369: Always includes Eyra (provides std support)
pub fn build_iso(arch: &str) -> Result<()> {
    build_iso_internal(&[], arch, false)
}

/// TEAM_286: Build ISO with verbose feature for behavior testing
pub fn build_iso_verbose(arch: &str) -> Result<()> {
    build_iso_internal(&["verbose"], arch, false)
}

/// TEAM_374: Build ISO for testing with test initramfs
pub fn build_iso_test(arch: &str) -> Result<()> {
    build_iso_internal(&["verbose"], arch, true)
}

fn build_iso_internal(features: &[&str], arch: &str, use_test_initramfs: bool) -> Result<()> {
    if arch != "x86_64" {
        bail!("ISO build currently only supported for x86_64");
    }

    println!("💿 Building Limine ISO for {}...", arch);

    // 1. Ensure all components are built
    // TEAM_369: Always build Eyra utilities (provides std)
    println!("🔧 Building Eyra utilities...");
    build_eyra(arch, None)?;
    build_userspace(arch)?;
    if use_test_initramfs {
        create_test_initramfs(arch)?;
    } else {
        create_initramfs(arch)?;
    }
    crate::disk::install_userspace_to_disk(arch)?;
    build_kernel_with_features(features, arch)?;

    let iso_root = PathBuf::from("iso_root");
    let boot_dir = iso_root.join("boot");
    
    // Clean and create staging directory
    if iso_root.exists() {
        std::fs::remove_dir_all(&iso_root)?;
    }
    std::fs::create_dir_all(&boot_dir)?;

    // 2. Copy components to ISO root
    let kernel_path = "crates/kernel/target/x86_64-unknown-none/release/levitate-kernel";
    // TEAM_374: Use test initramfs when in test mode
    let initramfs_path = if use_test_initramfs {
        "initramfs_test.cpio".to_string()
    } else {
        format!("initramfs_{}.cpio", arch)
    };
    let limine_cfg_path = "limine.cfg";

    std::fs::copy(kernel_path, boot_dir.join("levitate-kernel"))
        .context("Failed to copy levitate-kernel to ISO boot dir")?;
    if std::path::Path::new(&initramfs_path).exists() {
        std::fs::copy(&initramfs_path, boot_dir.join("initramfs.cpio"))
            .context("Failed to copy initramfs to ISO boot dir")?;
    }
    std::fs::copy(limine_cfg_path, iso_root.join("limine.cfg"))
        .context("Failed to copy limine.cfg - ensure it exists in repo root")?;

    // 3. Download/Prepare Limine binaries if needed
    prepare_limine_binaries(&iso_root)?;

    // 4. Create ISO using xorriso
    let iso_file = "levitate.iso";
    let status = Command::new("xorriso")
        .args([
            "-as", "mkisofs",
            "-b", "limine-bios-cd.bin",
            "-no-emul-boot", "-boot-load-size", "4", "-boot-info-table",
            "--efi-boot", "limine-uefi-cd.bin",
            "-efi-boot-part", "--efi-boot-image", "--protective-msdos-label",
            &iso_root.to_string_lossy(),
            "-o", iso_file,
        ])
        .status()
        .context("Failed to run xorriso")?;

    if !status.success() {
        bail!("xorriso failed to create ISO");
    }

    println!("✅ ISO created: {}", iso_file);
    Ok(())
}

/// TEAM_380: Build Eyra-based userspace utilities
/// Now uses uutils-coreutils multi-call binary instead of individual wrapper crates
pub fn build_eyra(arch: &str, _only: Option<&str>) -> Result<()> {
    let coreutils_dir = PathBuf::from("crates/userspace/eyra/coreutils");
    let eyra_dir = PathBuf::from("crates/userspace/eyra");
    
    if !coreutils_dir.exists() {
        bail!("Coreutils submodule not found: {}. Run 'git submodule update --init'", coreutils_dir.display());
    }
    
    let target = match arch {
        "x86_64" => "x86_64-unknown-linux-gnu",
        "aarch64" => "aarch64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}. Use 'x86_64' or 'aarch64'", arch),
    };
    
    println!("🔧 Building uutils-coreutils for {}...", arch);
    
    // TEAM_380: Build coreutils multi-call binary with pure-Rust utilities
    // We use a custom feature list excluding utilities that require C dependencies (onig)
    // Excluded: expr (requires onig for regex)
    let features = [
        // Basic file operations
        "cat", "cp", "dd", "ln", "ls", "mkdir", "mv", "rm", "rmdir", "touch",
        // Text processing (excluding expr which needs onig)
        "basename", "comm", "csplit", "cut", "dirname", "echo", "expand", 
        "fmt", "fold", "head", "join", "nl", "numfmt", "od", "paste",
        "pr", "printenv", "printf", "ptx", "seq", "shuf", "sort", "split",
        "sum", "tac", "tail", "tr", "truncate", "tsort", "unexpand", "uniq", "wc",
        // File utilities
        "dir", "dircolors", "df", "du", "link", "mktemp", "more", "readlink",
        "realpath", "shred", "sleep", "tee", "test", "unlink", "vdir", "yes",
        // Encoding
        "base32", "base64", "basenc",
        // Checksums
        "cksum", "hashsum",
        // Misc
        "env", "factor", "false", "pwd", "true",
    ].join(",");
    
    let status = Command::new("cargo")
        .current_dir(&coreutils_dir)
        .arg("+nightly-2025-04-28")
        .env_remove("RUSTUP_TOOLCHAIN")
        .args([
            "build",
            "--release",
            "--target", target,
            "--no-default-features",
            "--features", &features,
        ])
        .status()
        .context("Failed to build coreutils")?;
    
    if !status.success() {
        bail!("Failed to build coreutils");
    }
    
    println!("✅ Built coreutils multi-call binary");
    
    // TEAM_380: Also build eyra-hello and eyra-test-runner from the eyra workspace
    // TEAM_404: Do NOT use -Zbuild-std here! Eyra uses rename pattern (std = { package = "eyra" })
    // which provides its own std crate. -Zbuild-std would build Rust's real std, causing conflicts.
    println!("🔧 Building eyra-hello and eyra-test-runner...");
    let status = Command::new("cargo")
        .current_dir(&eyra_dir)
        .arg("+nightly-2025-04-28")
        .env_remove("RUSTUP_TOOLCHAIN")
        .args([
            "build",
            "--release",
            "--target", target,
        ])
        .status()
        .context("Failed to build eyra workspace")?;
    
    if !status.success() {
        bail!("Failed to build eyra workspace");
    }
    
    let out_dir = coreutils_dir.join(format!("target/{}/release", target));
    println!("📁 Coreutils binary: {}/coreutils", out_dir.display());
    
    Ok(())
}

fn prepare_limine_binaries(iso_root: &PathBuf) -> Result<()> {
    let limine_dir = PathBuf::from("limine-bin");
    let files = [
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
        "limine-bios.sys",
    ];
    
    // TEAM_304: Check if all required files exist, not just directory
    let all_files_exist = files.iter().all(|f| limine_dir.join(f).exists());
    
    if !all_files_exist {
        println!("📥 Downloading Limine binaries (v7.x)...");
        std::fs::create_dir_all(&limine_dir)?;
        
        let base_url = "https://github.com/limine-bootloader/limine/raw/v7.x-binary/";

        for file in &files {
            let url = format!("{}{}", base_url, file);
            let output = limine_dir.join(file);
            println!("  Fetching {}...", file);
            
            let status = Command::new("curl")
                .args(["-L", "-f", "-o", output.to_str().unwrap(), &url])
                .status()
                .context(format!("Failed to run curl for {}", file))?;
            
            if !status.success() {
                bail!("Failed to download {} from {}", file, url);
            }
        }
    }

    // Copy to ISO root for xorriso
    for file in &files {
        let src = limine_dir.join(file);
        let dst = iso_root.join(file);
        std::fs::copy(&src, &dst)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;
    }

    Ok(())
}
