use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::disk;

#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (Kernel + Userspace + Disk)
    All,
    /// Build kernel only
    Kernel,
    /// Build userspace only
    Userspace {
        /// Build specific package? (Not implemented, builds workspace)
        #[arg(long)]
        package: Option<String>,
    },
    /// Build initramfs only
    Initramfs,
    /// Build bootable Limine ISO
    Iso,
}

pub fn build_all(arch: &str) -> Result<()> {
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
    
    // TEAM_354: Add eyra-hello if it exists (static-pie test binary)
    let eyra_target = match arch {
        "aarch64" => "aarch64-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        _ => "",
    };
    let eyra_src = PathBuf::from(format!("userspace/eyra-hello/target/{}/release/eyra-hello", eyra_target));
    if eyra_src.exists() {
        std::fs::copy(&eyra_src, root.join("eyra-hello"))?;
        count += 1;
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

/// TEAM_243: Create test-specific initramfs with test_runner as init.
/// This initramfs boots directly into the test runner instead of shell.
pub fn create_test_initramfs(arch: &str) -> Result<()> {
    println!("Creating test initramfs for {}...", arch);
    let root = PathBuf::from("initrd_test_root");
    
    // Clean and create directory
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Copy test_runner as "init" - this is the key difference
    let test_runner_src = PathBuf::from(format!("crates/userspace/target/{}/release/test_runner", target));
    if !test_runner_src.exists() {
        bail!("test_runner binary not found - build userspace first");
    }
    std::fs::copy(&test_runner_src, root.join("init"))?;

    // Copy all test binaries (*_test)
    // TEAM_256: Added suite_test_core and core utilities it depends on
    let test_binaries = [
        // Test binaries
        "mmap_test", "pipe_test", "signal_test", "clone_test", "interrupt_test", 
        "tty_test", "pty_test", "pty_interact", "stat_test", "link_test", 
        "time_test", "sched_yield_test", "error_test",
        // TEAM_256: suite_test_core (core utils test suite)
        "suite_test_core",
        // TEAM_256: Core utilities spawned by suite_test_core
        "mkdir", "ls", "touch", "rm", "cp", "mv", "rmdir", "cat",
    ];
    let mut count = 0;
    for bin in &test_binaries {
        let src = PathBuf::from(format!("crates/userspace/target/{}/release/{}", target, bin));
        if src.exists() {
            std::fs::copy(&src, root.join(bin))?;
            count += 1;
        } else {
            println!("  Warning: {} not found", bin);
        }
    }
    println!("📦 Test initramfs: init + {} test binaries", count);

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
    if arch == "aarch64" {
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
    } else {
        // x86_64 uses multiboot2 (ELF) directly or needs different conversion
        println!("x86_64 kernel build complete (ELF format for multiboot2)");
    }

    Ok(())
}

/// TEAM_283: Build a bootable Limine ISO
pub fn build_iso(arch: &str) -> Result<()> {
    build_iso_with_features(&[], arch)
}

/// TEAM_286: Build ISO with verbose feature for behavior testing
pub fn build_iso_verbose(arch: &str) -> Result<()> {
    build_iso_with_features(&["verbose"], arch)
}

fn build_iso_with_features(features: &[&str], arch: &str) -> Result<()> {
    if arch != "x86_64" {
        bail!("ISO build currently only supported for x86_64");
    }

    println!("💿 Building Limine ISO for {}...", arch);

    // 1. Ensure all components are built
    build_userspace(arch)?;
    create_initramfs(arch)?;
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
    let kernel_path = "target/x86_64-unknown-none/release/levitate-kernel";
    // TEAM_327: Use arch-specific initramfs path
    let initramfs_path = format!("initramfs_{}.cpio", arch);
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
