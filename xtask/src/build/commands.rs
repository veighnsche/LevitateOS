use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Stdio};
use crate::disk;

// TEAM_435: Removed Eyra command, added Sysroot and Coreutils
#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (Kernel + Userspace + Disk + Coreutils)
    All,
    /// Build kernel only
    Kernel,
    /// Build userspace only
    Userspace,
    /// Build initramfs only
    Initramfs,
    /// Build bootable Limine ISO (includes coreutils)
    Iso,
    /// Build c-gull sysroot (libc.a)
    Sysroot,
    /// Build coreutils against sysroot
    Coreutils,
    /// Build brush shell against sysroot
    Brush,
}

// TEAM_435: Replaced Eyra with c-gull sysroot approach
pub fn build_all(arch: &str) -> Result<()> {
    // Build sysroot and coreutils if not present
    if !super::sysroot::sysroot_exists() {
        println!("🔧 Building c-gull sysroot...");
        super::sysroot::build_sysroot(arch)?;
    }
    if !super::external::coreutils_exists(arch) {
        println!("🔧 Building coreutils...");
        super::external::build_coreutils(arch)?;
    }

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

// TEAM_435: Uses c-gull sysroot binaries instead of Eyra
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

    // 2. Copy userspace binaries (init, shell - bare-metal)
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

    // TEAM_435: c-gull target for Linux binaries
    let linux_target = match arch {
        "aarch64" => "aarch64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        _ => "",
    };

    // c-gull libc test binary (built with standalone c-gull sysroot)
    let hello_cgull_src = PathBuf::from(format!("toolchain/libc-levitateos/test/rust-test/target/{}/release/hello_rust", linux_target));
    if hello_cgull_src.exists() {
        std::fs::copy(&hello_cgull_src, root.join("hello-cgull"))?;
        count += 1;
        println!("  📦 Added hello-cgull (c-gull libc test)");
    }

    // TEAM_435: Copy coreutils from new toolchain location
    let coreutils_src = PathBuf::from(format!(
        "toolchain/coreutils-out/{}/release/coreutils",
        linux_target
    ));

    // Utilities available with current c-gull (limited - missing getpwuid, etc.)
    let utils = [
        "cat", "echo", "head", "mkdir", "pwd", "rm", "tail", "touch",
    ];

    if coreutils_src.exists() {
        // Copy the multi-call binary
        std::fs::copy(&coreutils_src, root.join("coreutils"))?;
        count += 1;

        // Create symlinks for each utility
        for util in &utils {
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
        println!("  📦 Added coreutils + {} symlinks", utils.len());
    } else {
        println!("  ⚠️  Coreutils not found at {}", coreutils_src.display());
        println!("      Run 'cargo xtask build coreutils' first");
    }

    // TEAM_435: Add brush shell if built
    let brush_src = PathBuf::from(format!("toolchain/brush-out/{}/release/brush", linux_target));
    if brush_src.exists() {
        std::fs::copy(&brush_src, root.join("brush"))?;
        count += 1;
        println!("  📦 Added brush shell");
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

/// TEAM_435: Create test-specific initramfs with coreutils.
/// Includes init, shell, and coreutils for testing.
pub fn create_test_initramfs(arch: &str) -> Result<()> {
    println!("Creating test initramfs for {}...", arch);
    let root = PathBuf::from("initrd_test_root");

    // Clean and create directory
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    let linux_target = match arch {
        "aarch64" => "aarch64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let bare_target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Copy init and shell for boot
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

    // TEAM_435: Copy coreutils from toolchain location
    let coreutils_src = PathBuf::from(format!(
        "toolchain/coreutils-out/{}/release/coreutils",
        linux_target
    ));

    // Utilities available with current c-gull
    let utils = [
        "cat", "echo", "head", "mkdir", "pwd", "rm", "tail", "touch",
    ];
    let mut count = 0;

    if coreutils_src.exists() {
        // Copy the multi-call binary
        std::fs::copy(&coreutils_src, root.join("coreutils"))?;
        count += 1;

        // Create symlinks for each utility
        for util in &utils {
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
        bail!("Coreutils not found. Run 'cargo xtask build coreutils' first.");
    }

    println!("📦 Test initramfs: coreutils + {} symlinks + init/shell", utils.len());

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
// TEAM_435: Replaced Eyra with c-gull sysroot
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

    // TEAM_435: Build sysroot and coreutils if not present
    if !super::sysroot::sysroot_exists() {
        println!("🔧 Building c-gull sysroot...");
        super::sysroot::build_sysroot(arch)?;
    }
    if !super::external::coreutils_exists(arch) {
        println!("🔧 Building coreutils...");
        super::external::build_coreutils(arch)?;
    }

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

// TEAM_435: build_eyra() removed - replaced by build::external::build_coreutils()

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
