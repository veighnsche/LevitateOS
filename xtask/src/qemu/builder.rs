//! QEMU command builder
//!
//! TEAM_322: Builder pattern to eliminate duplicated QEMU arg construction.

use super::QemuProfile;
use anyhow::{bail, Result};
use std::process::Command;

/// Target architecture
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arch {
    Aarch64,
    X86_64,
}

impl Arch {
    /// Returns the QEMU binary name for this architecture
    pub fn qemu_binary(&self) -> &'static str {
        match self {
            Arch::Aarch64 => "qemu-system-aarch64",
            Arch::X86_64 => "qemu-system-x86_64",
        }
    }

    /// Returns the kernel binary path for this architecture
    pub fn kernel_path(&self) -> &'static str {
        match self {
            Arch::Aarch64 => "kernel64_rust.bin",
            Arch::X86_64 => "target/x86_64-unknown-none/release/levitate-kernel",
        }
    }

    /// Returns the virtio device suffix for this architecture
    fn device_suffix(&self) -> &'static str {
        match self {
            Arch::Aarch64 => "device",
            Arch::X86_64 => "pci",
        }
    }
}

impl TryFrom<&str> for Arch {
    type Error = anyhow::Error;

    fn try_from(s: &str) -> Result<Self> {
        match s {
            "aarch64" => Ok(Arch::Aarch64),
            "x86_64" => Ok(Arch::X86_64),
            _ => bail!("Unsupported architecture: {}", s),
        }
    }
}

/// Boot mode configuration
#[derive(Clone, Copy, Debug, Default)]
pub enum BootMode {
    /// Boot from kernel + initrd
    #[default]
    Kernel,
    /// Boot from ISO (Limine)
    Iso,
}

/// Display configuration
#[derive(Clone, Copy, Debug, Default)]
pub enum DisplayMode {
    /// GTK window
    #[default]
    Gtk,
    /// VNC server on :0
    Vnc,
    /// Headless (display=none)
    Headless,
    /// Nographic (-nographic, serial on stdio)
    Nographic,
}

/// GPU resolution configuration
#[derive(Clone, Copy, Debug)]
pub struct GpuResolution {
    pub width: u32,
    pub height: u32,
}

impl Default for GpuResolution {
    fn default() -> Self {
        Self { width: 1920, height: 1080 }
    }
}

/// QEMU command builder with fluent API
#[derive(Clone, Debug)]
pub struct QemuBuilder {
    arch: Arch,
    profile: QemuProfile,
    boot_mode: BootMode,
    display: DisplayMode,
    gpu_resolution: GpuResolution,
    // Features
    enable_gdb: bool,
    gdb_wait: bool,
    enable_qmp: bool,
    qmp_socket: String,
    enable_gpu_debug: bool,
    no_reboot: bool,
    // Disk
    disk_image: Option<String>,
    initrd: Option<String>,
}

impl QemuBuilder {
    /// Create a new QEMU builder for the given architecture and profile
    pub fn new(arch: Arch, profile: QemuProfile) -> Self {
        Self {
            arch,
            profile,
            boot_mode: BootMode::default(),
            display: DisplayMode::default(),
            gpu_resolution: GpuResolution::default(),
            enable_gdb: false,
            gdb_wait: false,
            enable_qmp: false,
            qmp_socket: "./qmp.sock".to_string(),
            enable_gpu_debug: false,
            no_reboot: true,
            disk_image: Some("tinyos_disk.img".to_string()),
            initrd: Some("initramfs.cpio".to_string()),
        }
    }

    /// Set boot mode to ISO
    pub fn boot_iso(mut self) -> Self {
        self.boot_mode = BootMode::Iso;
        self
    }

    /// Set boot mode to kernel with custom initrd
    pub fn boot_kernel(mut self, initrd: &str) -> Self {
        self.boot_mode = BootMode::Kernel;
        self.initrd = Some(initrd.to_string());
        self
    }

    /// Set display to GTK
    pub fn display_gtk(mut self) -> Self {
        self.display = DisplayMode::Gtk;
        self
    }

    /// Set display to VNC
    pub fn display_vnc(mut self) -> Self {
        self.display = DisplayMode::Vnc;
        self
    }

    /// Set display to headless
    pub fn display_headless(mut self) -> Self {
        self.display = DisplayMode::Headless;
        self
    }

    /// Set display to nographic (serial on stdio)
    pub fn display_nographic(mut self) -> Self {
        self.display = DisplayMode::Nographic;
        self
    }

    /// Set GPU resolution
    pub fn gpu_resolution(mut self, width: u32, height: u32) -> Self {
        self.gpu_resolution = GpuResolution { width, height };
        self
    }

    /// Enable GDB server on port 1234
    pub fn enable_gdb(mut self, wait: bool) -> Self {
        self.enable_gdb = true;
        self.gdb_wait = wait;
        self
    }

    /// Enable QMP socket
    pub fn enable_qmp(mut self, socket: &str) -> Self {
        self.enable_qmp = true;
        self.qmp_socket = socket.to_string();
        self
    }

    /// Enable GPU debug tracing
    pub fn enable_gpu_debug(mut self) -> Self {
        self.enable_gpu_debug = true;
        self
    }

    /// Set disk image path (None to disable)
    #[allow(dead_code)]
    pub fn disk_image(mut self, path: Option<&str>) -> Self {
        self.disk_image = path.map(|s| s.to_string());
        self
    }

    /// Set no-reboot flag
    #[allow(dead_code)]
    pub fn no_reboot(mut self, enabled: bool) -> Self {
        self.no_reboot = enabled;
        self
    }

    /// Build the QEMU command
    pub fn build(self) -> Result<Command> {
        let mut cmd = Command::new(self.arch.qemu_binary());
        let machine = self.profile.machine();

        // Machine and CPU
        cmd.args(["-M", &machine]);
        cmd.args(["-cpu", self.profile.cpu()]);
        cmd.args(["-m", self.profile.memory()]);

        // SMP if specified
        if let Some(smp) = self.profile.smp() {
            cmd.args(["-smp", smp]);
        }

        // Boot configuration
        match self.boot_mode {
            BootMode::Kernel => {
                cmd.args(["-kernel", self.arch.kernel_path()]);
                if let Some(ref initrd) = self.initrd {
                    cmd.args(["-initrd", initrd]);
                }
            }
            BootMode::Iso => {
                cmd.args(["-cdrom", "levitate.iso", "-boot", "d"]);
            }
        }

        // GPU device
        let gpu_spec = format!(
            "virtio-gpu-pci,xres={},yres={}",
            self.gpu_resolution.width, self.gpu_resolution.height
        );
        cmd.args(["-device", &gpu_spec]);

        // Input devices (arch-specific suffix)
        let suffix = self.arch.device_suffix();
        cmd.args(["-device", &format!("virtio-keyboard-{}", suffix)]);
        cmd.args(["-device", &format!("virtio-tablet-{}", suffix)]);

        // Network
        let net_device = format!("virtio-net-{},netdev=net0", suffix);
        cmd.args(["-device", &net_device]);
        cmd.args(["-netdev", "user,id=net0"]);

        // Disk
        if let Some(ref disk) = self.disk_image {
            cmd.args(["-drive", &format!("file={},format=raw,if=none,id=hd0", disk)]);
            cmd.args(["-device", &format!("virtio-blk-{},drive=hd0", suffix)]);
        }

        // Display
        match self.display {
            DisplayMode::Gtk => {
                cmd.args(["-display", "gtk,zoom-to-fit=off,window-close=off"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
            DisplayMode::Vnc => {
                cmd.args(["-display", "none"]);
                cmd.args(["-vnc", ":0"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
            DisplayMode::Headless => {
                cmd.args(["-display", "none"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
            DisplayMode::Nographic => {
                cmd.args(["-nographic"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
        }

        // GDB
        if self.enable_gdb {
            cmd.arg("-s"); // Shorthand for -gdb tcp::1234
            if self.gdb_wait {
                cmd.arg("-S"); // Freeze CPU at startup
            }
        }

        // QMP
        if self.enable_qmp {
            cmd.args(["-qmp", &format!("unix:{},server,nowait", self.qmp_socket)]);
        }

        // GPU debug
        if self.enable_gpu_debug {
            cmd.args(["-d", "guest_errors"]);
            cmd.args(["-D", "qemu_gpu_debug.log"]);
        }

        // No reboot
        if self.no_reboot {
            cmd.arg("-no-reboot");
        }

        Ok(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_from_str() {
        assert!(matches!(Arch::try_from("aarch64"), Ok(Arch::Aarch64)));
        assert!(matches!(Arch::try_from("x86_64"), Ok(Arch::X86_64)));
        assert!(Arch::try_from("arm").is_err());
    }

    #[test]
    fn test_builder_basic() {
        let builder = QemuBuilder::new(Arch::X86_64, QemuProfile::X86_64);
        let cmd = builder.build().unwrap();
        let args: Vec<_> = cmd.get_args().collect();
        
        // Should contain machine type
        assert!(args.iter().any(|a| a.to_str() == Some("q35")));
        // Should contain cpu
        assert!(args.iter().any(|a| a.to_str() == Some("qemu64")));
    }
}
