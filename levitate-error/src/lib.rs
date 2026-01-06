//! TEAM_155: Kernel error handling infrastructure.
//!
//! Provides the `define_kernel_error!` macro for consistent error type definitions.
//!
//! ## Usage
//!
//! ### Simple errors (no inner data)
//! ```ignore
//! define_kernel_error! {
//!     pub enum NetError(0x07) {
//!         NotInitialized = 0x01 => "Network device not initialized",
//!         DeviceBusy = 0x02 => "TX queue full",
//!     }
//! }
//! ```
//!
//! ### Nested errors (with inner error type)
//! ```ignore
//! define_kernel_error! {
//!     pub enum SpawnError(0x03) {
//!         Elf(ElfError) = 0x01 => "ELF loading failed",
//!         PageTable(MmuError) = 0x02 => "Page table creation failed",
//!     }
//! }
//! ```

#![no_std]

/// Macro to define a kernel error type with consistent handling.
///
/// Supports both simple variants and nested variants containing inner errors.
#[macro_export]
macro_rules! define_kernel_error {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident($subsystem:literal) {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $(($inner:ty))? = $code:literal => $desc:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant $(($inner))?,
            )*
        }

        impl $name {
            /// Subsystem identifier for this error type.
            pub const SUBSYSTEM: u8 = $subsystem;

            /// Get numeric error code for debugging.
            pub const fn code(&self) -> u16 {
                match self {
                    $(
                        $crate::define_kernel_error!(@pattern $variant $(($inner))? _unused) => {
                            (($subsystem as u16) << 8) | $code
                        }
                    )*
                }
            }

            /// Get error name for logging.
            pub const fn name(&self) -> &'static str {
                match self {
                    $(
                        $crate::define_kernel_error!(@pattern $variant $(($inner))? _unused) => {
                            $desc
                        }
                    )*
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(
                        $crate::define_kernel_error!(@pattern $variant $(($inner))? inner) => {
                            $crate::define_kernel_error!(@display_body self f $desc $(($inner))? inner)
                        }
                    )*
                }
            }
        }

        impl core::error::Error for $name {}
    };

    // Helper to generate patterns
    (@pattern $variant:ident ($inner:ty) $bind:ident) => { Self::$variant($bind) };
    (@pattern $variant:ident $bind:ident) => { Self::$variant };

    // Helper to generate display bodies
    (@display_body $self:ident $f:ident $desc:literal ($inner:ty) $bind:ident) => {
        write!($f, "E{:04X}: {} ({})", $self.code(), $desc, $bind)
    };
    (@display_body $self:ident $f:ident $desc:literal $bind:ident) => {
        write!($f, "E{:04X}: {}", $self.code(), $desc)
    };
}

#[cfg(test)]
mod tests {

    define_kernel_error! {
        /// Test error type
        pub enum TestError(0xFF) {
            /// First error
            First = 0x01 => "First error",
            /// Second error
            Second = 0x02 => "Second error",
        }
    }

    define_kernel_error! {
        pub enum NestedTestError(0xFE) {
            Inner(TestError) = 0x01 => "Nested error",
        }
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(TestError::First.code(), 0xFF01);
        assert_eq!(TestError::Second.code(), 0xFF02);
        assert_eq!(NestedTestError::Inner(TestError::First).code(), 0xFE01);
    }

    #[test]
    fn test_error_names() {
        assert_eq!(TestError::First.name(), "First error");
        assert_eq!(TestError::Second.name(), "Second error");
        assert_eq!(
            NestedTestError::Inner(TestError::First).name(),
            "Nested error"
        );
    }

    #[test]
    fn test_display_format() {
        // Simple
        extern crate std;
        use std::format;
        assert_eq!(format!("{}", TestError::First), "EFF01: First error");

        // Nested
        let inner = TestError::First;
        assert_eq!(
            format!("{}", NestedTestError::Inner(inner)),
            "EFE01: Nested error (EFF01: First error)"
        );
    }

    #[test]
    fn test_subsystem_constant() {
        assert_eq!(TestError::SUBSYSTEM, 0xFF);
        assert_eq!(NestedTestError::SUBSYSTEM, 0xFE);
    }

    #[test]
    fn test_derives() {
        // Clone
        let e = TestError::First;
        let e2 = e.clone();
        assert_eq!(e, e2);

        // Copy
        let e3 = e;
        assert_eq!(e, e3);

        // Debug
        extern crate std;
        use std::format;
        let debug_str = format!("{:?}", TestError::First);
        assert!(debug_str.contains("First"));
    }
}
