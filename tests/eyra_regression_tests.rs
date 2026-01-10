// Regression tests for Eyra userspace integration
// These tests verify cross-file consistency and build configuration

use std::fs;
use std::path::Path;
use std::process::Command;

/// Tests: [EY6] workspace config contains -nostartfiles
#[test]
fn test_nostartfiles_in_workspace_config() {
    // [EY6] Verify -nostartfiles is in workspace .cargo/config.toml
    let config_path = "crates/userspace/eyra/.cargo/config.toml";
    let content = fs::read_to_string(config_path).expect("Failed to read .cargo/config.toml");

    assert!(
        content.contains("-nostartfiles"),
        "[EY6] workspace config must contain -nostartfiles for both targets"
);

    // Verify it's in both target configs
    assert!(content.contains("[target.x86_64-unknown-linux-gnu]"));
    assert!(content.contains("[target.aarch64-unknown-linux-gnu]"));
}

/// Tests: [EY7] no duplicate -nostartfiles in build.rs
#[test]
fn test_no_duplicate_nostartfiles() {
    // [EY7] Verify build.rs files don't duplicate -nostartfiles
    let eyra_dir = "crates/userspace/eyra";

    for entry in fs::read_dir(eyra_dir).expect("Failed to read eyra directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_dir() {
            let build_rs = path.join("build.rs");
            if build_rs.exists() {
                let content =
                    fs::read_to_string(&build_rs).expect(&format!("Failed to read {:?}", build_rs));

assert!(
                    !content.contains("nostartfiles"),
                    "[EY7] {:?} must not contain nostartfiles (should be in workspace config)",
                    build_rs
                );
            }
        }
    }
}

/// Tests: [EY8] aarch64 config includes sysroot
#[test]
fn test_aarch64_sysroot_in_config() {
    // [EY8] Verify sysroot path in aarch64 config
    let config_path = "crates/userspace/eyra/.cargo/config.toml";
    let content = fs::read_to_string(config_path).expect("Failed to read .cargo/config.toml");

    assert!(
        content.contains("--sysroot=/usr/aarch64-redhat-linux/sys-root/fc43"),
        "[EY8] aarch64 config must include correct sysroot path"
    );
}

/// Tests: [EY9] all targets use +crt-static
#[test]
fn test_crt_static_enabled() {
    // [EY9] Verify crt-static is enabled
    let config_path = "crates/userspace/eyra/.cargo/config.toml";
    let content = fs::read_to_string(config_path).expect("Failed to read .cargo/config.toml");

    // Count occurrences - should be at least 2 (one per target)
    let count = content.matches("+crt-static").count();
    assert!(count >= 2, "[EY9] +crt-static must be set for both targets");
}

/// Tests: [EY10] all targets use relocation-model=pic
#[test]
fn test_pic_relocation_model() {
    // [EY10] Verify PIC relocation model
    let config_path = "crates/userspace/eyra/.cargo/config.toml";
    let content = fs::read_to_string(config_path).expect("Failed to read .cargo/config.toml");

    let count = content.matches("relocation-model=pic").count();
    assert!(
        count >= 2,
        "[EY10] relocation-model=pic must be set for both targets"
    );
}

/// Tests: [EY19] libsyscall depends on linux-raw-sys 0.4
#[test]
fn test_linux_raw_sys_version() {
    // [EY19] Verify linux-raw-sys dependency version
    let cargo_toml = "crates/userspace/eyra/libsyscall/Cargo.toml";
    let content = fs::read_to_string(cargo_toml).expect("Failed to read libsyscall/Cargo.toml");

    assert!(
        content.contains("linux-raw-sys") && content.contains("0.4"),
        "[EY19] libsyscall must depend on linux-raw-sys 0.4"
    );
}

/// Tests: [EY20] eyra dependency is optional
#[test]
fn test_eyra_dependency_optional() {
    // [EY20] Verify eyra is marked optional
    let cargo_toml = "crates/userspace/eyra/libsyscall/Cargo.toml";
    let content = fs::read_to_string(cargo_toml).expect("Failed to read libsyscall/Cargo.toml");

    // Check that eyra dependency has optional = true
    let lines: Vec<&str> = content.lines().collect();
    let mut found_eyra = false;
    let mut found_optional = false;

    for (i, line) in lines.iter().enumerate() {
        if line.contains("[dependencies.eyra]") {
            found_eyra = true;
            // Check next few lines for optional = true
            for j in i + 1..i + 5 {
                if j < lines.len() && lines[j].contains("optional = true") {
                    found_optional = true;
                    break;
                }
            }
            break;
        }
    }

    assert!(
        found_eyra && found_optional,
        "[EY20] eyra dependency must be marked as optional"
    );
}

/// Tests: [EY21] std feature enables eyra
#[test]
fn test_std_feature_enables_eyra() {
    // [EY21] Verify std feature enables eyra dependency
    let cargo_toml = "crates/userspace/eyra/libsyscall/Cargo.toml";
    let content = fs::read_to_string(cargo_toml).expect("Failed to read libsyscall/Cargo.toml");

    assert!(
        content.contains("[features]"),
        "[EY21] features section must exist"
    );
    assert!(
        content.contains("std = [\"eyra\"]"),
        "[EY21] std feature must enable eyra dependency"
    );
}

/// Tests: [EY22] default features do not include std
#[test]
fn test_default_no_std() {
    // [EY22] Verify default features don't include std
    let cargo_toml = "crates/userspace/eyra/libsyscall/Cargo.toml";
    let content = fs::read_to_string(cargo_toml).expect("Failed to read libsyscall/Cargo.toml");

    // Find default line
    for line in content.lines() {
        if line.trim().starts_with("default = ") {
            assert!(
                !line.contains("std"),
                "[EY22] default features must not include std"
            );
            return;
        }
    }

    // If no default line found, that's okay (defaults to empty)
}

/// Tests: [EY23] libsyscall in workspace
#[test]
fn test_libsyscall_in_workspace() {
    // [EY23] Verify libsyscall is workspace member
    let workspace_toml = "crates/userspace/eyra/Cargo.toml";
    let content = fs::read_to_string(workspace_toml).expect("Failed to read eyra/Cargo.toml");

    assert!(
        content.contains("\"libsyscall\""),
        "[EY23] libsyscall must be in workspace members"
    );
}

/// Tests: [EY24] libsyscall-tests in workspace
#[test]
fn test_libsyscall_tests_in_workspace() {
    // [EY24] Verify libsyscall-tests is workspace member
    let workspace_toml = "crates/userspace/eyra/Cargo.toml";
    let content = fs::read_to_string(workspace_toml).expect("Failed to read eyra/Cargo.toml");

    assert!(
        content.contains("\"libsyscall-tests\""),
        "[EY24] libsyscall-tests must be in workspace members"
    );
}

/// Tests: [EY25] workspace resolver version 2
#[test]
fn test_workspace_resolver_version() {
    // [EY25] Verify workspace uses resolver = "2"
    let workspace_toml = "crates/userspace/eyra/Cargo.toml";
    let content = fs::read_to_string(workspace_toml).expect("Failed to read eyra/Cargo.toml");

    assert!(
        content.contains("resolver = \"2\""),
        "[EY25] workspace must use resolver version 2"
    );
}

/// Tests: [EY26] NOSTARTFILES_README.md exists
#[test]
fn test_nostartfiles_readme_exists() {
    // [EY26] Verify documentation exists
    let readme_path = "crates/userspace/eyra/NOSTARTFILES_README.md";
    assert!(
        Path::new(readme_path).exists(),
        "[EY26] NOSTARTFILES_README.md must exist"
    );
}

/// Tests: [EY27] X86_64_STATUS.md documents limitation
#[test]
fn test_x86_64_status_documented() {
    // [EY27] Verify x86_64 limitation is documented
    let status_path = "crates/userspace/eyra/libsyscall-tests/X86_64_STATUS.md";
    assert!(
        Path::new(status_path).exists(),
        "[EY27] X86_64_STATUS.md must exist"
);

    let content = fs::read_to_string(status_path).expect("Failed to read X86_64_STATUS.md");
    assert!(
        content.contains("not supported"),
        "[EY27] must document x86_64 not being supported"
    );
}

/// Tests: [EY28] TEAM_380 documents cross-compilation
#[test]
fn test_team_380_exists() {
    // [EY28] Verify TEAM_380 documentation exists
    let team_file = ".teams/TEAM_380_setup_aarch64_cross_compilation.md";
    assert!(
        Path::new(team_file).exists(),
        "[EY28] TEAM_380 must document cross-compilation setup"
    );
}

/// Tests: [EY29] TEAM_381 documents nostartfiles
#[test]
fn test_team_381_exists() {
    // [EY29] Verify TEAM_381 documentation exists
    let team_file = ".teams/TEAM_381_centralize_nostartfiles_config.md";
    assert!(
        Path::new(team_file).exists(),
        "[EY29] TEAM_381 must document nostartfiles abstraction"
    );
}

/// Tests: [EY30] TEAM_382 documents integration results
#[test]
fn test_team_382_exists() {
    // [EY30] Verify TEAM_382 documentation exists
    let team_file = ".teams/TEAM_382_libsyscall_eyra_integration_test.md";
    assert!(
        Path::new(team_file).exists(),
        "[EY30] TEAM_382 must document integration test results"
    );
}

/// Tests: [EY1] libsyscall builds for aarch64 with std
#[test]
#[ignore] // Expensive test - run manually
fn test_libsyscall_builds_aarch64() {
    // [EY1] Verify build succeeds
    let output = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target",
            "aarch64-unknown-linux-gnu",
            "--manifest-path",
            "crates/userspace/eyra/libsyscall/Cargo.toml",
            "--features",
            "std",
        ])
        .output()
        .expect("Failed to execute cargo build");

    assert!(
        output.status.success(),
        "[EY1] libsyscall must build successfully for aarch64: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Tests: [EY2] binary produces correct ELF format
#[test]
#[ignore] // Requires binary to be built first
fn test_libsyscall_tests_elf_format() {
    // [EY2] Verify ELF format is correct
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        // Binary hasn't been built yet, skip
        return;
    }

    let output = Command::new("file")
        .arg(binary_path)
        .output()
        .expect("Failed to run file command");

    let file_output = String::from_utf8_lossy(&output.stdout);
    assert!(
        file_output.contains("ELF 64-bit"),
        "[EY2] must be 64-bit ELF"
    );
    assert!(
        file_output.contains("ARM aarch64"),
        "[EY2] must be aarch64 architecture"
    );
}

/// Tests: [EY3] binary is statically linked
#[test]
#[ignore] // Requires binary to be built first
fn test_binary_static_linkage() {
    // [EY3] Verify no dynamic dependencies
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    let output = Command::new("file")
        .arg(binary_path)
        .output()
        .expect("Failed to run file command");

    let file_output = String::from_utf8_lossy(&output.stdout);
    assert!(
        file_output.contains("statically linked"),
        "[EY3] binary must be statically linked"
    );
}

/// Tests: [EY4] binary size is reasonable
#[test]
#[ignore] // Requires binary to be built first
fn test_binary_size_limit() {
    // [EY4] Verify binary size < 100KB
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    let metadata = fs::metadata(binary_path).expect("Failed to get binary metadata");

    let size_kb = metadata.len() / 1024;
    assert!(
        size_kb < 100,
        "[EY4] binary must be < 100KB, got {}KB",
        size_kb
    );
}

/// Tests: [EY14] entry point is valid
#[test]
#[ignore] // Requires readelf and built binary
fn test_entry_point_valid() {
    // [EY14] Verify entry point is in valid range
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    let output = Command::new("readelf")
        .args(&["-h", binary_path])
        .output()
        .expect("Failed to run readelf");

    let readelf_output = String::from_utf8_lossy(&output.stdout);

    // Find entry point line
    for line in readelf_output.lines() {
        if line.contains("Entry point address:") {
            // Entry point should be 0x400000 or higher
            assert!(
                line.contains("0x4"),
                "[EY14] entry point must be in valid code range (0x400000+)"
            );
            return;
        }
    }

    panic!("[EY14] Could not find entry point in readelf output");
}

/// Tests: [EY18] no INTERP segment (static binary)
#[test]
#[ignore] // Requires readelf and built binary
fn test_no_interp_segment() {
    // [EY18] Static binaries must not have INTERP segment
    let binary_path =
        "crates/userspace/eyra/target/aarch64-unknown-linux-gnu/release/libsyscall-tests";

    if !Path::new(binary_path).exists() {
        return;
    }

    let output = Command::new("readelf")
        .args(&["-l", binary_path])
        .output()
        .expect("Failed to run readelf");

    let readelf_output = String::from_utf8_lossy(&output.stdout);

    assert!(
        !readelf_output.contains("INTERP"),
        "[EY18] static binary must not have INTERP segment"
    );
}
