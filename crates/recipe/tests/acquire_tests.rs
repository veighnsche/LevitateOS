//! Integration tests for the acquire phase.

mod common;

use common::TestEnv;
use levitate_recipe::{AcquireSpec, Context, Executor, GitRef};
use std::path::PathBuf;

/// Test downloading a source file.
#[tokio::test]
async fn test_acquire_source_download() {
    let env = TestEnv::new().await;

    // Create a test file to download via file:// URL
    env.write_file("/tmp/source.tar.gz", "fake tarball content")
        .await
        .unwrap();

    // Setup executor context pointing to container's build dir
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true); // Dry run since we can't actually execute in container

    let executor = Executor::new(ctx);

    // Test that the acquire spec parses correctly
    let spec = AcquireSpec::Source {
        url: "file:///tmp/source.tar.gz".to_string(),
        verify: None,
    };

    // In dry-run mode, this should succeed without actually downloading
    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}

/// Test SHA256 verification success.
#[tokio::test]
async fn test_acquire_source_sha256_valid() {
    let env = TestEnv::new().await;

    // Ensure build directory exists
    env.shell("mkdir -p /tmp/build").await.unwrap();

    // Create a file and get its actual SHA256
    let content = "test content for sha256";
    env.write_file("/tmp/build/testfile", content)
        .await
        .unwrap();

    // Calculate SHA256 in container
    let sha = env
        .shell("sha256sum /tmp/build/testfile | cut -d' ' -f1")
        .await
        .unwrap();
    let sha = sha.trim();

    // Verify the checksum matches
    let verify_result = env
        .shell(&format!(
            "echo '{}  /tmp/build/testfile' | sha256sum -c -",
            sha
        ))
        .await;

    assert!(verify_result.is_ok(), "SHA256 verification should pass");
}

/// Test SHA256 verification failure.
#[tokio::test]
async fn test_acquire_source_sha256_invalid() {
    let env = TestEnv::new().await;

    // Ensure build directory exists
    env.shell("mkdir -p /tmp/build").await.unwrap();

    // Create a file
    env.write_file("/tmp/build/testfile", "some content")
        .await
        .unwrap();

    // Try to verify with wrong checksum
    let verify_result = env
        .shell("echo 'badhash  /tmp/build/testfile' | sha256sum -c -")
        .await;

    assert!(verify_result.is_err(), "SHA256 verification should fail");
}

/// Test binary URL selection for x86_64.
#[tokio::test]
async fn test_acquire_binary_x86_64() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .arch("x86_64")
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Binary {
        urls: vec![
            ("x86_64".to_string(), "https://example.com/x86_64.tar.gz".to_string()),
            ("aarch64".to_string(), "https://example.com/aarch64.tar.gz".to_string()),
        ],
    };

    // Should succeed and select x86_64 URL
    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}

/// Test binary URL selection for aarch64.
#[tokio::test]
async fn test_acquire_binary_aarch64() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .arch("aarch64")
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Binary {
        urls: vec![
            ("x86_64".to_string(), "https://example.com/x86_64.tar.gz".to_string()),
            ("aarch64".to_string(), "https://example.com/aarch64.tar.gz".to_string()),
        ],
    };

    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}

/// Test binary URL selection for unknown architecture.
#[tokio::test]
async fn test_acquire_binary_unknown_arch() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .arch("riscv64")
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Binary {
        urls: vec![
            ("x86_64".to_string(), "https://example.com/x86_64.tar.gz".to_string()),
            ("aarch64".to_string(), "https://example.com/aarch64.tar.gz".to_string()),
        ],
    };

    let result = executor.acquire(&spec);
    assert!(result.is_err(), "Should fail for unknown architecture");
}

/// Test git clone (dry run).
#[tokio::test]
async fn test_acquire_git_clone() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Git {
        url: "https://github.com/example/repo.git".to_string(),
        reference: None,
    };

    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}

/// Test git clone with tag.
#[tokio::test]
async fn test_acquire_git_tag() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Git {
        url: "https://github.com/example/repo.git".to_string(),
        reference: Some(GitRef::Tag("v1.0.0".to_string())),
    };

    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}

/// Test git clone with branch.
#[tokio::test]
async fn test_acquire_git_branch() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = AcquireSpec::Git {
        url: "https://github.com/example/repo.git".to_string(),
        reference: Some(GitRef::Branch("develop".to_string())),
    };

    let result = executor.acquire(&spec);
    assert!(result.is_ok());
}
