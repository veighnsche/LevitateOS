//! QEMU hardware profiles
//!
//! `TEAM_322`: Extracted from run.rs

/// QEMU hardware profiles for different target configurations
#[derive(Clone, Copy, Debug, Default)]
pub enum QemuProfile {
    /// Default: 2GB RAM, 1 core, cortex-a53 (aarch64)
    #[default]
    Default,
    /// Pixel 6: 8GB RAM, 8 cores, cortex-a76, `GICv3`
    Pixel6,
    /// Test: `GICv3` on default machine
    GicV3,
    /// `x86_64`: 2GB RAM, 1 core, q35
    X86_64,
}

impl QemuProfile {
    /// Returns the QEMU machine type string
    pub fn machine(&self) -> String {
        match self {
            QemuProfile::Default => "virt".to_string(),
            QemuProfile::Pixel6 => "virt,gic-version=3".to_string(),
            QemuProfile::GicV3 => "virt,gic-version=3".to_string(),
            QemuProfile::X86_64 => "q35".to_string(),
        }
    }

    /// Returns the CPU model
    pub fn cpu(&self) -> &'static str {
        match self {
            QemuProfile::Default => "cortex-a53",
            QemuProfile::Pixel6 => "cortex-a76",
            QemuProfile::GicV3 => "cortex-a53",
            QemuProfile::X86_64 => "qemu64",
        }
    }

    /// Returns the memory size
    pub fn memory(&self) -> &'static str {
        match self {
            // TEAM_387: Increased from 512M to 2G for mature coreutils support
            QemuProfile::Default => "2G",
            QemuProfile::Pixel6 => "8G",
            QemuProfile::GicV3 => "2G",
            QemuProfile::X86_64 => "2G",
        }
    }

    /// Returns optional SMP topology string
    pub fn smp(&self) -> Option<&'static str> {
        match self {
            QemuProfile::Default => None,
            QemuProfile::Pixel6 => Some("8"),
            QemuProfile::GicV3 => None,
            QemuProfile::X86_64 => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_machine() {
        assert_eq!(QemuProfile::Default.machine(), "virt");
    }

    #[test]
    fn test_x86_64_machine() {
        assert_eq!(QemuProfile::X86_64.machine(), "q35");
    }

    #[test]
    fn test_pixel6_smp() {
        assert_eq!(QemuProfile::Pixel6.smp(), Some("8"));
    }

    #[test]
    fn test_default_no_smp() {
        assert_eq!(QemuProfile::Default.smp(), None);
    }

    #[test]
    fn test_memory_values() {
        assert_eq!(QemuProfile::Default.memory(), "2G");
        assert_eq!(QemuProfile::Pixel6.memory(), "8G");
        assert_eq!(QemuProfile::X86_64.memory(), "2G");
    }

    #[test]
    fn test_cpu_values() {
        assert_eq!(QemuProfile::Default.cpu(), "cortex-a53");
        assert_eq!(QemuProfile::Pixel6.cpu(), "cortex-a76");
        assert_eq!(QemuProfile::X86_64.cpu(), "qemu64");
    }
}
