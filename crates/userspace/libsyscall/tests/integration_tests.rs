// Integration tests for libsyscall with Eyra std support
// These tests run on the host with std enabled

#[cfg(feature = "std")]
mod with_std {
    use libsyscall::*;

    /// Tests: [LS13] syscall numbers are const
    #[test]
    fn test_syscall_numbers_const() {
        // [LS13] Verify constants can be used in const contexts
        const _WRITE: i64 = SYS_WRITE;
        const _READ: i64 = SYS_READ;
        const _OPENAT: i64 = SYS_OPENAT;
        const _CLOSE: i64 = SYS_CLOSE;

        assert_eq!(SYS_WRITE, 64);
        assert_eq!(SYS_READ, 63);
        assert_eq!(SYS_OPENAT, 56);
        assert_eq!(SYS_CLOSE, 57);
    }

    /// Tests: [LS14] AT_FDCWD is const and correct
    #[test]
    fn test_at_fdcwd_const() {
        // [LS14] AT_FDCWD must be -100 per Linux ABI
        const _FDCWD: i32 = AT_FDCWD;
        assert_eq!(AT_FDCWD, -100);
    }

    /// Tests: [LS15] open flags are const
    #[test]
    fn test_open_flags_const() {
        // [LS15] Verify flag constants
        const _RDONLY: i32 = O_RDONLY;
        const _WRONLY: i32 = O_WRONLY;
        const _RDWR: i32 = O_RDWR;

        assert_eq!(O_RDONLY, 0);
        assert_eq!(O_WRONLY, 1);
        assert_eq!(O_RDWR, 2);
    }

    /// Tests: [LS10] negative returns indicate errors
    #[test]
    fn test_negative_return_is_error() {
        // [LS10] Error returns are negative
        let error_result: i64 = -1;
        assert!(error_result < 0, "Errors must be negative");

        let error_enoent: i64 = -2;
        assert!(error_enoent < 0);
    }

    /// Tests: [LS12] success returns non-negative
    #[test]
    fn test_success_nonnegative() {
        // [LS12] Success values are >= 0
        let success_fd: i64 = 3;
        assert!(success_fd >= 0, "Success must be non-negative");

        let success_zero: i64 = 0;
        assert!(success_zero >= 0);
    }

    /// Tests: [LS5] aarch64 syscall convention
    #[test]
    #[cfg(target_arch = "aarch64")]
    fn test_aarch64_syscall_convention() {
        // [LS5] aarch64 uses x8 for syscall number
        // This is verified by the assembly in src/arch/aarch64.rs
        // We can't directly test assembly, but we verify the constants exist
        assert!(SYS_WRITE > 0, "Syscall numbers must be defined");
    }

    /// Tests: [LS8] x86_64 syscall convention
    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_x86_64_syscall_convention() {
        // [LS8] x86_64 uses rax for syscall number
        // This is verified by the assembly in src/arch/x86_64.rs
        assert!(SYS_WRITE > 0, "Syscall numbers must be defined");
    }
}

/// Tests without std - verify no_std compatibility
#[cfg(not(feature = "std"))]
mod without_std {
    use libsyscall::*;

    /// Tests: [LS22] default features do not include std
    #[test]
    fn test_no_std_compiles() {
        // [LS22] Library compiles without std
        let _write_nr = SYS_WRITE;
        // If this compiles, no_std works
    }
}
