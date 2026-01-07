use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::image;

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
}

pub fn build_all(arch: &str) -> Result<()> {
    // TEAM_073: Build userspace first
    build_userspace(arch)?;
    create_initramfs(arch)?;
    // TEAM_121: Ensure disk image is populated
    image::install_userspace_to_disk(arch)?;

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
        .current_dir("userspace")
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
    if !root.exists() {
        std::fs::create_dir(&root)?;
    }

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
        let src = PathBuf::from(format!("userspace/target/{}/release/{}", target, bin));
        if src.exists() {
            std::fs::copy(&src, root.join(bin))?;
            count += 1;
        }
    }
    println!("[DONE] ({} added)", count);

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
    let test_runner_src = PathBuf::from(format!("userspace/target/{}/release/test_runner", target));
    if !test_runner_src.exists() {
        bail!("test_runner binary not found - build userspace first");
    }
    std::fs::copy(&test_runner_src, root.join("init"))?;

    // Copy all test binaries (*_test)
    let test_binaries = ["mmap_test", "pipe_test", "signal_test", "clone_test", "interrupt_test", "tty_test", "pty_test", "pty_interact", "stat_test", "link_test", "time_test", "sched_yield_test", "error_test"];
    let mut count = 0;
    for bin in &test_binaries {
        let src = PathBuf::from(format!("userspace/target/{}/release/{}", target, bin));
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
