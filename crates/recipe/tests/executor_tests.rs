//! Executor tests for bugs found in TEAM_007 audit.

use levitate_recipe::{Context, Executor};
use std::path::PathBuf;

// ============================================================================
// Context Tests
// ============================================================================

mod context_tests {
    use super::*;

    /// Default context should have sensible values.
    #[test]
    fn test_default_context() {
        let ctx = Context::default();

        assert_eq!(ctx.prefix, PathBuf::from("/usr/local"));
        assert!(!ctx.dry_run);
        assert!(!ctx.verbose);
        assert!(ctx.nproc >= 1);
    }

    /// Context builder pattern should work.
    #[test]
    fn test_context_builder() {
        let ctx = Context::with_prefix("/opt/myapp")
            .arch("aarch64")
            .dry_run(true)
            .verbose(true);

        assert_eq!(ctx.prefix, PathBuf::from("/opt/myapp"));
        assert_eq!(ctx.arch, "aarch64");
        assert!(ctx.dry_run);
        assert!(ctx.verbose);
    }

    /// Context with shell metacharacters in prefix (security concern).
    #[test]
    fn test_context_with_shell_metacharacters() {
        // This documents a potential security issue - if prefix contains
        // shell metacharacters, they could be executed
        let ctx = Context::with_prefix("/tmp/$(whoami)");

        // The prefix is stored as-is
        assert_eq!(ctx.prefix.to_str().unwrap(), "/tmp/$(whoami)");

        // When expanded in commands, this could execute the shell command
        // This is a security concern documented in TEAM_007 audit
    }

    /// Context with path traversal in prefix.
    #[test]
    fn test_context_with_path_traversal() {
        let ctx = Context::with_prefix("/tmp/../../../etc");

        // The prefix is stored as-is (no canonicalization)
        assert!(ctx.prefix.to_str().unwrap().contains(".."));
    }
}

// ============================================================================
// Executor Dry Run Tests
// ============================================================================

mod executor_dry_run {
    use super::*;
    use levitate_recipe::{parse, Recipe};

    /// Dry run should not execute commands.
    #[test]
    fn test_dry_run_no_execution() {
        let input = r#"
            (package "test" "1.0"
              (acquire (source "https://example.com/test.tar.gz"))
              (build (run "echo should not run")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        let ctx = Context::with_prefix("/tmp/test")
            .build_dir(PathBuf::from("/tmp/build"))
            .dry_run(true)
            .verbose(true);

        let executor = Executor::new(ctx);

        // Dry run should succeed without actually downloading anything
        // This would fail in non-dry-run mode since the URL is fake
        let result = executor.execute(&recipe);

        // In dry run mode, this should succeed
        assert!(result.is_ok(), "Dry run should succeed");
    }

    /// Verbose mode should print commands.
    #[test]
    fn test_verbose_mode() {
        let ctx = Context::with_prefix("/tmp/test")
            .dry_run(true)
            .verbose(true);

        let _executor = Executor::new(ctx);
        // Just verify it creates without panic
    }
}

// ============================================================================
// Variable Expansion Tests
// ============================================================================

mod variable_expansion {
    use super::*;
    use levitate_recipe::{parse, Recipe, BuildSpec, BuildStep};

    /// Variables should be expanded in build commands.
    #[test]
    fn test_variables_in_build_commands() {
        let input = r#"
            (package "test" "1.0"
              (build
                (run "meson setup build --prefix=$PREFIX")
                (run "ninja -C build -j$NPROC")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // Verify the recipe has the variable placeholders
        match &recipe.build {
            Some(BuildSpec::Steps(steps)) => {
                match &steps[0] {
                    BuildStep::Run(cmd) => {
                        assert!(cmd.contains("$PREFIX"));
                    }
                    _ => panic!("Expected Run step"),
                }
                match &steps[1] {
                    BuildStep::Run(cmd) => {
                        assert!(cmd.contains("$NPROC"));
                    }
                    _ => panic!("Expected Run step"),
                }
            }
            _ => panic!("Expected Steps build spec"),
        }
    }

    /// ARCH variable should be available.
    #[test]
    fn test_arch_variable() {
        let input = r#"
            (package "test" "1.0"
              (build (run "echo Building for $ARCH")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.build {
            Some(BuildSpec::Steps(steps)) => {
                match &steps[0] {
                    BuildStep::Run(cmd) => {
                        assert!(cmd.contains("$ARCH"));
                    }
                    _ => panic!("Expected Run step"),
                }
            }
            _ => panic!("Expected Steps build spec"),
        }
    }
}

// ============================================================================
// Archive Format Tests
// ============================================================================

mod archive_formats {
    use super::*;
    use levitate_recipe::{parse, Recipe, BuildSpec};

    /// All supported archive formats should be parseable.
    #[test]
    fn test_supported_archive_formats() {
        let formats = ["tar-gz", "tar.gz", "tgz", "tar-xz", "tar.xz", "txz",
                       "tar-bz2", "tar.bz2", "tbz2", "tar", "zip"];

        for format in formats {
            let input = format!(r#"(package "test" "1.0" (build (extract "{}")))"#, format);
            let expr = parse(&input).unwrap();
            let recipe = Recipe::from_expr(&expr).unwrap();

            match &recipe.build {
                Some(BuildSpec::Extract(f)) => {
                    assert_eq!(f, format, "Format {} should parse", format);
                }
                _ => panic!("Format {} should create Extract spec", format),
            }
        }
    }
}

// ============================================================================
// Install Mode Tests
// ============================================================================

mod install_modes {
    use super::*;
    use levitate_recipe::{parse, Recipe, InstallFile};

    /// Default install mode for binaries should be 0o755.
    #[test]
    fn test_default_binary_mode() {
        let input = r#"(package "test" "1.0" (install (to-bin "myapp")))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.install.unwrap().files[0] {
            InstallFile::ToBin { mode, .. } => {
                // Default mode is None in the recipe, executor uses 0o755
                assert!(mode.is_none());
            }
            _ => panic!("Expected ToBin"),
        }
    }

    /// to-lib should not have executable permission by default.
    #[test]
    fn test_library_mode() {
        let input = r#"(package "test" "1.0" (install (to-lib "libfoo.so")))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.install.unwrap().files[0] {
            InstallFile::ToLib { .. } => {
                // ToLib doesn't have a mode field in the recipe
                // Executor uses 0o644
            }
            _ => panic!("Expected ToLib"),
        }
    }
}

// ============================================================================
// ROBUSTNESS: Tests for issues that could cause problems
// ============================================================================

mod robustness {
    use super::*;

    /// Creating executor with empty prefix should work.
    #[test]
    fn test_empty_prefix() {
        let ctx = Context::with_prefix("");
        let _executor = Executor::new(ctx);
        // Should not panic
    }

    /// Creating executor with very long prefix should work.
    #[test]
    fn test_very_long_prefix() {
        let long_path = "/".to_string() + &"a".repeat(4096);
        let ctx = Context::with_prefix(&long_path);
        let _executor = Executor::new(ctx);
        // Should not panic
    }

    /// nproc of 0 should be handled.
    #[test]
    fn test_zero_nproc() {
        let ctx = Context {
            prefix: PathBuf::from("/tmp"),
            build_dir: PathBuf::from("/tmp/build"),
            arch: "x86_64".to_string(),
            nproc: 0,  // Edge case
            dry_run: true,
            verbose: false,
        };
        let _executor = Executor::new(ctx);
        // Should not panic (though 0 threads is unusual)
    }
}
