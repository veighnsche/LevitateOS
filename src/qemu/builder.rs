//! QEMU command builder
//!
//! `TEAM_322`: Builder pattern to eliminate duplicated QEMU arg construction.

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

    /// Returns the custom kernel binary path for this architecture
    pub fn kernel_path(&self) -> &'static str {
        match self {
            Arch::Aarch64 => "kernel64_rust.bin",
            Arch::X86_64 => "crates/kernel/target/x86_64-unknown-none/release/levitate-kernel",
        }
    }

    /// TEAM_474: Returns the Linux kernel binary path for this architecture
    pub fn linux_kernel_path(&self) -> &'static str {
        match self {
            Arch::Aarch64 => "linux/arch/arm64/boot/Image",
            Arch::X86_64 => "linux/arch/x86/boot/bzImage",
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
            _ => bail!("Unsupported architecture: {s}"),
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
        // TEAM_330: Use 1280x800 for readable text (128x36 char grid with 10x20 font)
        Self {
            width: 1280,
            height: 800,
        }
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
    // TEAM_474: Linux kernel support
    use_linux_kernel: bool,
    // TEAM_476: Serial output to file (for behavior tests)
    serial_file: Option<String>,
}

impl QemuBuilder {
    /// Create a new QEMU builder for the given architecture and profile
    pub fn new(arch: Arch, profile: QemuProfile) -> Self {
        // TEAM_327: Use arch-specific initramfs to prevent cross-arch contamination
        let initrd_name = match arch {
            Arch::Aarch64 => "initramfs_aarch64.cpio",
            Arch::X86_64 => "initramfs_x86_64.cpio",
        };
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
            initrd: Some(initrd_name.to_string()),
            use_linux_kernel: false,
            serial_file: None,
        }
    }

    /// TEAM_476: Output serial to file (for behavior tests)
    pub fn serial_file(mut self, path: &str) -> Self {
        self.serial_file = Some(path.to_string());
        self
    }

    /// TEAM_474: Use Linux kernel instead of custom kernel
    pub fn linux_kernel(mut self) -> Self {
        self.use_linux_kernel = true;
        self
    }

    /// TEAM_475: Set custom initramfs path
    pub fn initrd(mut self, path: &str) -> Self {
        self.initrd = Some(path.to_string());
        self
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
        self.disk_image = path.map(std::string::ToString::to_string);
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
                // TEAM_474: Use Linux or custom kernel based on flag
                let kernel_path = if self.use_linux_kernel {
                    self.arch.linux_kernel_path()
                } else {
                    self.arch.kernel_path()
                };
                cmd.args(["-kernel", kernel_path]);
                if let Some(ref initrd) = self.initrd {
                    cmd.args(["-initrd", initrd]);
                }
                // TEAM_474: Linux kernel needs command line for console
                if self.use_linux_kernel {
                    cmd.args(["-append", "console=ttyS0 earlyprintk=serial,ttyS0,115200 rdinit=/init"]);
                }
            }
            BootMode::Iso => {
                cmd.args(["-cdrom", "levitate.iso", "-boot", "d"]);
            }
        }

        // GPU device
        // TEAM_331: x86_64 display mode:
        // - GTK/SDL: Use -vga std (Limine framebuffer, proper window size)
        // - VNC/headless: Use virtio-gpu-pci (proper resolution via edid)
        // - Nographic: Use virtio-gpu but no VGA (TEAM_444)
        match (&self.arch, &self.display) {
            (Arch::X86_64, DisplayMode::Gtk) => {
                // Use VGA std - Limine gets framebuffer, GTK shows correct window size
                cmd.args(["-vga", "std"]);
            }
            (Arch::X86_64, DisplayMode::Nographic) => {
                // TEAM_444: For nographic, still add virtio-gpu for GPU init but no VGA
                cmd.args(["-vga", "none"]);
                let gpu_spec = format!(
                    "virtio-gpu-pci,xres={},yres={},edid=on",
                    self.gpu_resolution.width, self.gpu_resolution.height
                );
                cmd.args(["-device", &gpu_spec]);
            }
            (Arch::X86_64, _) => {
                // VNC/headless: use virtio-gpu for proper resolution
                cmd.args(["-vga", "none"]);
                let gpu_spec = format!(
                    "virtio-gpu-pci,xres={},yres={},edid=on",
                    self.gpu_resolution.width, self.gpu_resolution.height
                );
                cmd.args(["-device", &gpu_spec]);
            }
            _ => {
                // aarch64: always use virtio-gpu-pci
                let gpu_spec = format!(
                    "virtio-gpu-pci,xres={},yres={}",
                    self.gpu_resolution.width, self.gpu_resolution.height
                );
                cmd.args(["-device", &gpu_spec]);
            }
        }

        // Input devices (arch-specific suffix)
        // TEAM_444: Skip VirtIO input in nographic mode - use serial for input
        let suffix = self.arch.device_suffix();
        if !matches!(self.display, DisplayMode::Nographic) {
            cmd.args(["-device", &format!("virtio-keyboard-{suffix}")]);
            cmd.args(["-device", &format!("virtio-tablet-{suffix}")]);
        }

        // Network
        let net_device = format!("virtio-net-{suffix},netdev=net0");
        cmd.args(["-device", &net_device]);
        cmd.args(["-netdev", "user,id=net0"]);

        // Disk
        if let Some(ref disk) = self.disk_image {
            cmd.args(["-drive", &format!("file={disk},format=raw,if=none,id=hd0")]);
            cmd.args(["-device", &format!("virtio-blk-{suffix},drive=hd0")]);
        }

        // Display
        match self.display {
            DisplayMode::Gtk => {
                // TEAM_330: Use SDL instead of GTK - GTK ignores virtio-gpu resolution
                cmd.args(["-display", "sdl"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
            DisplayMode::Vnc => {
                cmd.args(["-display", "none"]);
                cmd.args(["-vnc", ":0"]);
                cmd.args(["-serial", "mon:stdio"]);
            }
            DisplayMode::Headless => {
                cmd.args(["-display", "none"]);
                // TEAM_476: Serial to file if set, otherwise stdio
                if let Some(ref file) = self.serial_file {
                    cmd.args(["-serial", &format!("file:{}", file)]);
                } else {
                    cmd.args(["-serial", "mon:stdio"]);
                }
            }
            DisplayMode::Nographic => {
                // TEAM_444: Simple serial on stdio without mux
                // mux=on with monitor causes input to go to monitor by default
                cmd.args(["-display", "none"]);
                cmd.args(["-serial", "stdio"]);
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

        // TEAM_409: Add isa-debug-exit device for x86_64 to allow proper VM termination
        // The kernel writes to port 0xf4 to exit QEMU
        if matches!(self.arch, Arch::X86_64) {
            cmd.args(["-device", "isa-debug-exit,iobase=0xf4,iosize=0x04"]);
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
