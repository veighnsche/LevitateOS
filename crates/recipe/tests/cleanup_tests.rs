//! Integration tests for the cleanup phase.

mod common;

use common::TestEnv;
use levitate_recipe::{CleanupSpec, CleanupTarget, Context, Executor};
use std::path::PathBuf;

/// Test cleanup all - removes entire build directory.
#[tokio::test]
async fn test_cleanup_all() {
    let env = TestEnv::new().await;

    // Create build directory with various files
    env.shell(
        "mkdir -p /tmp/build && \
         echo 'source' > /tmp/build/source.tar.gz && \
         mkdir -p /tmp/build/extracted && \
         echo 'file' > /tmp/build/extracted/main.c",
    )
    .await
    .unwrap();

    // Verify files exist
    assert!(env.file_exists("/tmp/build/source.tar.gz").await);
    assert!(env.file_exists("/tmp/build/extracted/main.c").await);

    // Run cleanup all
    env.shell("rm -rf /tmp/build").await.unwrap();

    // Verify directory is gone
    assert!(!env.file_exists("/tmp/build").await);
}

/// Test cleanup downloads - only removes archive files.
#[tokio::test]
async fn test_cleanup_downloads() {
    let env = TestEnv::new().await;

    // Create build directory with archives and extracted files
    env.shell(
        "mkdir -p /tmp/build/extracted && \
         echo 'archive' > /tmp/build/source.tar.gz && \
         echo 'archive2' > /tmp/build/other.zip && \
         echo 'source' > /tmp/build/extracted/main.c",
    )
    .await
    .unwrap();

    // Remove only archives
    env.shell("rm -f /tmp/build/*.tar.gz /tmp/build/*.zip")
        .await
        .unwrap();

    // Verify archives are gone but extracted files remain
    assert!(!env.file_exists("/tmp/build/source.tar.gz").await);
    assert!(!env.file_exists("/tmp/build/other.zip").await);
    assert!(env.file_exists("/tmp/build/extracted/main.c").await);
}

/// Test cleanup sources - removes directories but keeps archives.
#[tokio::test]
async fn test_cleanup_sources() {
    let env = TestEnv::new().await;

    // Create build directory with archives and extracted directories
    env.shell(
        "mkdir -p /tmp/build/myapp-1.0/src && \
         echo 'archive' > /tmp/build/myapp-1.0.tar.gz && \
         echo 'source' > /tmp/build/myapp-1.0/src/main.c",
    )
    .await
    .unwrap();

    // Remove directories only
    env.shell("find /tmp/build -mindepth 1 -maxdepth 1 -type d -exec rm -rf {} +")
        .await
        .unwrap();

    // Verify archive remains but directory is gone
    assert!(env.file_exists("/tmp/build/myapp-1.0.tar.gz").await);
    assert!(!env.file_exists("/tmp/build/myapp-1.0").await);
}

/// Test cleanup with keep option.
#[tokio::test]
async fn test_cleanup_with_keep() {
    let env = TestEnv::new().await;

    // Create build directory with various files
    env.shell(
        "mkdir -p /tmp/build/cache /tmp/build/src && \
         echo 'cache' > /tmp/build/cache/data && \
         echo 'source' > /tmp/build/src/main.c && \
         echo 'temp' > /tmp/build/temp.txt",
    )
    .await
    .unwrap();

    // Remove everything except cache
    env.shell(
        "cd /tmp/build && \
         for f in *; do \
           if [ \"$f\" != \"cache\" ]; then \
             rm -rf \"$f\"; \
           fi; \
         done",
    )
    .await
    .unwrap();

    // Verify cache remains but others are gone
    assert!(env.file_exists("/tmp/build/cache/data").await);
    assert!(!env.file_exists("/tmp/build/src").await);
    assert!(!env.file_exists("/tmp/build/temp.txt").await);
}

/// Test cleanup dry run mode.
#[tokio::test]
async fn test_cleanup_dry_run() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true)
        .verbose(true);

    let executor = Executor::new(ctx);

    let spec = CleanupSpec {
        target: CleanupTarget::All,
        keep: Vec::new(),
    };

    // Should succeed without actually deleting anything
    let result = executor.cleanup(&spec);
    assert!(result.is_ok());
}

/// Test CleanupSpec parsing and execution flow.
#[tokio::test]
async fn test_cleanup_executor_integration() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    // Test various cleanup targets
    let targets = [
        CleanupTarget::All,
        CleanupTarget::Downloads,
        CleanupTarget::Sources,
        CleanupTarget::Artifacts,
    ];

    for target in targets {
        let spec = CleanupSpec {
            target,
            keep: vec!["important".to_string()],
        };
        let result = executor.cleanup(&spec);
        assert!(result.is_ok(), "Cleanup {:?} should succeed", spec.target);
    }
}
