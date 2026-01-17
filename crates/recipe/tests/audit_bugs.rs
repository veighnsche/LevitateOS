//! Comprehensive tests for bugs found in TEAM_007 audit.
//!
//! These tests document and verify bugs found during the recipe crate audit.
//! Tests marked with #[should_panic] or that assert bug behavior will need
//! to be updated once the bugs are fixed.

use levitate_recipe::{parse, Recipe, AcquireSpec, BuildSpec, Verify, GitRef};

// ============================================================================
// BUG #1: Dependency Parsing Broken (CRITICAL)
// Location: recipe.rs:245-258
// ============================================================================

mod dependency_parsing {
    use super::*;

    /// The nested deps format used in all recipe files is NOT parsed correctly.
    /// This test documents the bug: deps should be ["wayland"] but are [].
    #[test]
    fn test_nested_deps_format_is_broken() {
        let input = r#"
            (package "sway" "1.10"
              (deps
                (build "meson" "ninja" "pkg-config")
                (runtime "wlroots" "wayland")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // BUG: These should NOT be empty!
        // When fixed, change these to assert!(!recipe.deps.is_empty())
        assert!(recipe.deps.is_empty(), "BUG #1: nested deps are silently ignored");
        assert!(recipe.build_deps.is_empty(), "BUG #1: nested build deps are silently ignored");
    }

    /// The flat deps format (that the parser expects) does work.
    #[test]
    fn test_flat_deps_format_works() {
        let input = r#"
            (package "test" "1.0"
              (deps wayland wlroots)
              (build-deps meson ninja))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        assert_eq!(recipe.deps, vec!["wayland", "wlroots"]);
        assert_eq!(recipe.build_deps, vec!["meson", "ninja"]);
    }

    /// Empty deps section should work.
    #[test]
    fn test_empty_deps() {
        let input = r#"(package "test" "1.0" (deps))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();
        assert!(recipe.deps.is_empty());
    }
}

// ============================================================================
// BUG #2: SHA256 Verification Not Parsed (HIGH)
// Location: recipe.rs:306-313
// ============================================================================

mod sha256_parsing {
    use super::*;

    /// SHA256 verification in nested format is not parsed.
    #[test]
    fn test_sha256_nested_format_is_broken() {
        let input = r#"
            (package "wayland" "1.23.0"
              (acquire
                (source "https://example.com/wayland-1.23.0.tar.xz"
                  (sha256 "05b3c0b00504f2d952b9eb1aec56a7d8a870b1e1fd1e83bcf46c660d9b97e0e4"))))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.acquire {
            Some(AcquireSpec::Source { url, verify }) => {
                assert_eq!(url, "https://example.com/wayland-1.23.0.tar.xz");
                // BUG: verify should be Some(Verify::Sha256(...))
                assert!(verify.is_none(), "BUG #2: SHA256 verification is not parsed");
            }
            _ => panic!("Expected Source acquire spec"),
        }
    }

    /// Basic source URL without verification should work.
    #[test]
    fn test_source_without_sha256() {
        let input = r#"
            (package "test" "1.0"
              (acquire
                (source "https://example.com/test.tar.gz")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.acquire {
            Some(AcquireSpec::Source { url, verify }) => {
                assert_eq!(url, "https://example.com/test.tar.gz");
                assert!(verify.is_none());
            }
            _ => panic!("Expected Source acquire spec"),
        }
    }
}

// ============================================================================
// BUG #3: Git References Not Parsed (MEDIUM)
// Location: recipe.rs:329-335
// ============================================================================

mod git_reference_parsing {
    use super::*;

    /// Git tag reference is not parsed.
    #[test]
    fn test_git_tag_not_parsed() {
        let input = r#"
            (package "test" "1.0"
              (acquire
                (git "https://github.com/test/repo.git" (tag "v1.0"))))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.acquire {
            Some(AcquireSpec::Git { url, reference }) => {
                assert_eq!(url, "https://github.com/test/repo.git");
                // BUG: reference should be Some(GitRef::Tag("v1.0"))
                assert!(reference.is_none(), "BUG #3: Git tag reference not parsed");
            }
            _ => panic!("Expected Git acquire spec"),
        }
    }

    /// Git branch reference is not parsed.
    #[test]
    fn test_git_branch_not_parsed() {
        let input = r#"
            (package "test" "1.0"
              (acquire
                (git "https://github.com/test/repo.git" (branch "main"))))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.acquire {
            Some(AcquireSpec::Git { url, reference }) => {
                // BUG: reference should be Some(GitRef::Branch("main"))
                assert!(reference.is_none(), "BUG #3: Git branch reference not parsed");
            }
            _ => panic!("Expected Git acquire spec"),
        }
    }

    /// Git commit reference is not parsed.
    #[test]
    fn test_git_commit_not_parsed() {
        let input = r#"
            (package "test" "1.0"
              (acquire
                (git "https://github.com/test/repo.git" (commit "abc123"))))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.acquire {
            Some(AcquireSpec::Git { url, reference }) => {
                // BUG: reference should be Some(GitRef::Commit("abc123"))
                assert!(reference.is_none(), "BUG #3: Git commit reference not parsed");
            }
            _ => panic!("Expected Git acquire spec"),
        }
    }
}

// ============================================================================
// BUG #5: Empty Package Name/Version Accepted (MEDIUM)
// Location: recipe.rs:177-185
// ============================================================================

mod validation {
    use super::*;

    /// Empty package name should be rejected.
    #[test]
    fn test_empty_package_name_accepted_bug() {
        let input = r#"(package "" "1.0")"#;
        let expr = parse(input).unwrap();
        let result = Recipe::from_expr(&expr);

        // BUG: This should return Err, not Ok
        assert!(result.is_ok(), "BUG #5: Empty package name is accepted");
        if let Ok(recipe) = result {
            assert!(recipe.name.is_empty());
        }
    }

    /// Empty version should be rejected.
    #[test]
    fn test_empty_version_accepted_bug() {
        let input = r#"(package "test" "")"#;
        let expr = parse(input).unwrap();
        let result = Recipe::from_expr(&expr);

        // BUG: This should return Err, not Ok
        assert!(result.is_ok(), "BUG #5: Empty version is accepted");
        if let Ok(recipe) = result {
            assert!(recipe.version.is_empty());
        }
    }

    /// Missing version should be rejected (this works correctly).
    #[test]
    fn test_missing_version_rejected() {
        let input = r#"(package "test")"#;
        let expr = parse(input).unwrap();
        let result = Recipe::from_expr(&expr);

        assert!(result.is_err(), "Missing version should be rejected");
    }

    /// Missing name should be rejected (this works correctly).
    #[test]
    fn test_missing_name_rejected() {
        let input = r#"(package)"#;
        let expr = parse(input).unwrap();
        let result = Recipe::from_expr(&expr);

        assert!(result.is_err(), "Missing name should be rejected");
    }

    /// Empty source URL should be rejected.
    #[test]
    fn test_empty_source_url_accepted_bug() {
        let input = r#"(package "test" "1.0" (acquire (source "")))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // BUG: Should reject empty URL
        match &recipe.acquire {
            Some(AcquireSpec::Source { url, .. }) => {
                assert!(url.is_empty(), "BUG: Empty source URL is accepted");
            }
            _ => panic!("Expected Source acquire spec"),
        }
    }
}

// ============================================================================
// BUG #6: Duplicate Actions Silently Override (LOW)
// Location: recipe.rs:214-292
// ============================================================================

mod duplicate_actions {
    use super::*;

    /// Duplicate description actions - last one wins, no warning.
    #[test]
    fn test_duplicate_description_no_warning() {
        let input = r#"
            (package "test" "1.0"
              (description "first description")
              (description "second description"))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // Last one wins - this behavior could be documented or changed to error
        assert_eq!(recipe.description, Some("second description".to_string()));
    }

    /// Duplicate license actions.
    #[test]
    fn test_duplicate_license() {
        let input = r#"
            (package "test" "1.0"
              (license "MIT")
              (license "Apache-2.0" "GPL-3.0"))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // Last one wins
        assert_eq!(recipe.license, vec!["Apache-2.0", "GPL-3.0"]);
    }
}

// ============================================================================
// SECURITY #1: Path Traversal in Install (HIGH)
// Location: executor.rs:440-455
// ============================================================================

mod security_path_traversal {
    use super::*;

    /// Path traversal in to-bin is accepted (security bug).
    #[test]
    fn test_path_traversal_in_to_bin_accepted() {
        let input = r#"
            (package "test" "1.0"
              (install (to-bin "../../../etc/passwd")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // BUG: Should reject path traversal
        assert!(recipe.install.is_some(), "SECURITY: Path traversal accepted in recipe");
    }

    /// Absolute path in to-bin is accepted (potential security issue).
    #[test]
    fn test_absolute_path_in_to_bin_accepted() {
        let input = r#"
            (package "test" "1.0"
              (install (to-bin "/etc/shadow")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // Should potentially reject or warn about absolute paths
        assert!(recipe.install.is_some(), "SECURITY: Absolute path accepted");
    }
}

// ============================================================================
// Parser Edge Cases
// ============================================================================

mod parser_edge_cases {
    use super::*;

    /// Unknown action should be rejected.
    #[test]
    fn test_unknown_action_rejected() {
        let input = r#"(package "test" "1.0" (unknown-action "value"))"#;
        let expr = parse(input).unwrap();
        let result = Recipe::from_expr(&expr);

        assert!(result.is_err(), "Unknown action should be rejected");
    }

    /// Unicode in package name is accepted.
    #[test]
    fn test_unicode_package_name() {
        let input = r#"(package "tëst-пакет" "1.0")"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        assert_eq!(recipe.name, "tëst-пакет");
    }

    /// Deeply nested lists can be parsed (potential stack issue at extreme depths).
    #[test]
    fn test_nested_lists_100_deep() {
        let deep = "((".repeat(100) + "test" + &"))".repeat(100);
        let result = parse(&deep);

        // Should parse without stack overflow
        assert!(result.is_ok(), "Should handle 100-deep nesting");
    }

    /// String with escape sequences.
    #[test]
    fn test_string_escapes() {
        let input = r#"(package "test\n\t\"quoted\"" "1.0")"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        assert_eq!(recipe.name, "test\n\t\"quoted\"");
    }

    /// Comments should be ignored.
    #[test]
    fn test_comments_ignored() {
        let input = r#"
            ; This is a comment
            (package "test" "1.0"
              ; Another comment
              (description "desc"))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        assert_eq!(recipe.name, "test");
        assert_eq!(recipe.description, Some("desc".to_string()));
    }
}

// ============================================================================
// Build Phase Tests
// ============================================================================

mod build_phase {
    use super::*;

    /// Build skip should work.
    #[test]
    fn test_build_skip() {
        let input = r#"(package "test" "1.0" (build skip))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        assert!(matches!(recipe.build, Some(BuildSpec::Skip)));
    }

    /// Build extract should work.
    #[test]
    fn test_build_extract() {
        let input = r#"(package "test" "1.0" (build (extract "tar-gz")))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.build {
            Some(BuildSpec::Extract(format)) => {
                assert_eq!(format, "tar-gz");
            }
            _ => panic!("Expected Extract build spec"),
        }
    }

    /// Build with run commands.
    #[test]
    fn test_build_with_run() {
        let input = r#"
            (package "test" "1.0"
              (build
                (run "cd test && ./configure")
                (run "make -j$NPROC")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match &recipe.build {
            Some(BuildSpec::Steps(steps)) => {
                assert_eq!(steps.len(), 2);
            }
            _ => panic!("Expected Steps build spec"),
        }
    }
}

// ============================================================================
// Install Phase Tests
// ============================================================================

mod install_phase {
    use super::*;
    use levitate_recipe::InstallFile;

    /// Install to-bin with optional dest.
    #[test]
    fn test_install_to_bin() {
        let input = r#"
            (package "test" "1.0"
              (install
                (to-bin "build/myapp")
                (to-bin "build/helper" "helper-renamed")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        let install = recipe.install.unwrap();
        assert_eq!(install.files.len(), 2);

        match &install.files[0] {
            InstallFile::ToBin { src, dest, .. } => {
                assert_eq!(src, "build/myapp");
                assert!(dest.is_none());
            }
            _ => panic!("Expected ToBin"),
        }

        match &install.files[1] {
            InstallFile::ToBin { src, dest, .. } => {
                assert_eq!(src, "build/helper");
                assert_eq!(dest.as_deref(), Some("helper-renamed"));
            }
            _ => panic!("Expected ToBin"),
        }
    }

    /// Install to-lib.
    #[test]
    fn test_install_to_lib() {
        let input = r#"
            (package "test" "1.0"
              (install (to-lib "build/libfoo.so")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        let install = recipe.install.unwrap();
        match &install.files[0] {
            InstallFile::ToLib { src, .. } => {
                assert_eq!(src, "build/libfoo.so");
            }
            _ => panic!("Expected ToLib"),
        }
    }
}

// ============================================================================
// Cleanup Phase Tests
// ============================================================================

mod cleanup_phase {
    use super::*;
    use levitate_recipe::{CleanupSpec, CleanupTarget};

    /// Cleanup with no args defaults to all.
    #[test]
    fn test_cleanup_default() {
        let input = r#"(package "test" "1.0" (cleanup))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match recipe.cleanup {
            Some(CleanupSpec { target, keep }) => {
                assert!(matches!(target, CleanupTarget::All));
                assert!(keep.is_empty());
            }
            None => panic!("Expected cleanup spec"),
        }
    }

    /// Cleanup sources.
    #[test]
    fn test_cleanup_sources() {
        let input = r#"(package "test" "1.0" (cleanup sources))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match recipe.cleanup {
            Some(CleanupSpec { target, .. }) => {
                assert!(matches!(target, CleanupTarget::Sources));
            }
            None => panic!("Expected cleanup spec"),
        }
    }

    /// Cleanup with keep list.
    #[test]
    fn test_cleanup_with_keep() {
        let input = r#"(package "test" "1.0" (cleanup all (keep "cache" "logs")))"#;
        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        match recipe.cleanup {
            Some(CleanupSpec { target, keep }) => {
                assert!(matches!(target, CleanupTarget::All));
                assert_eq!(keep, vec!["cache", "logs"]);
            }
            None => panic!("Expected cleanup spec"),
        }
    }
}

// ============================================================================
// GAP Tests - Features that need implementation
// ============================================================================

mod gap_tests {
    use super::*;

    /// Configure phase is not implemented.
    #[test]
    fn test_configure_not_implemented() {
        let input = r#"
            (package "test" "1.0"
              (configure
                (create-user "testuser" system no-login)
                (create-dir "/var/lib/test")))
        "#;

        let expr = parse(input).unwrap();
        let recipe = Recipe::from_expr(&expr).unwrap();

        // GAP: Configure is not implemented, returns None
        assert!(recipe.configure.is_none(), "GAP: Configure not implemented");
    }
}

// ============================================================================
// Real Recipe File Tests
// ============================================================================

mod real_recipes {
    use super::*;
    use std::fs;

    /// Test that all example recipes at least parse without panic.
    #[test]
    fn test_all_example_recipes_parse() {
        let recipe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

        if !recipe_dir.exists() {
            eprintln!("Skipping: examples directory not found");
            return;
        }

        let mut parsed = 0;
        let mut errors = Vec::new();

        for entry in fs::read_dir(&recipe_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.extension().map(|e| e == "recipe").unwrap_or(false) {
                let content = fs::read_to_string(&path).unwrap();
                let name = path.file_name().unwrap().to_string_lossy();

                match parse(&content) {
                    Ok(expr) => match Recipe::from_expr(&expr) {
                        Ok(_) => parsed += 1,
                        Err(e) => errors.push(format!("{}: Recipe error: {}", name, e)),
                    },
                    Err(e) => errors.push(format!("{}: Parse error: {}", name, e)),
                }
            }
        }

        if !errors.is_empty() {
            panic!("Recipe errors:\n{}", errors.join("\n"));
        }

        assert!(parsed > 0, "No recipes found to test");
        eprintln!("Successfully parsed {} recipes", parsed);
    }

    /// Test that sway.recipe parses (even though deps are broken).
    #[test]
    fn test_sway_recipe_parses() {
        let recipe_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
        let sway_path = recipe_dir.join("sway.recipe");

        if !sway_path.exists() {
            eprintln!("Skipping: sway.recipe not found");
            return;
        }

        let content = fs::read_to_string(&sway_path).unwrap();
        let expr = parse(&content).expect("sway.recipe should parse");
        let recipe = Recipe::from_expr(&expr).expect("sway.recipe should be valid");

        assert_eq!(recipe.name, "sway");
        assert_eq!(recipe.version, "1.10");

        // Document the bug: deps should not be empty
        assert!(recipe.deps.is_empty(), "BUG: sway deps are not parsed");
        assert!(recipe.build_deps.is_empty(), "BUG: sway build_deps are not parsed");
    }
}
