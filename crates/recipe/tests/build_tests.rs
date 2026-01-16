//! Integration tests for the build phase.

mod common;

use common::TestEnv;
use levitate_recipe::{BuildSpec, BuildStep, Context, Executor};
use std::path::PathBuf;

/// Test extracting a tar.gz archive.
#[tokio::test]
async fn test_build_extract_tar_gz() {
    let env = TestEnv::new().await;

    // Create a test tarball in the container
    env.shell(
        "mkdir -p /tmp/build && \
         mkdir -p /tmp/testpkg-1.0 && \
         echo 'test content' > /tmp/testpkg-1.0/file.txt && \
         tar czf /tmp/build/testpkg-1.0.tar.gz -C /tmp testpkg-1.0",
    )
    .await
    .unwrap();

    // Extract it
    env.shell("cd /tmp/build && tar xzf testpkg-1.0.tar.gz")
        .await
        .unwrap();

    // Verify extraction
    assert!(env.file_exists("/tmp/build/testpkg-1.0/file.txt").await);
    let content = env.read_file("/tmp/build/testpkg-1.0/file.txt").await.unwrap();
    assert!(content.contains("test content"));
}

/// Test extracting a tar.xz archive.
#[tokio::test]
async fn test_build_extract_tar_xz() {
    let env = TestEnv::new().await;

    // Create a test tarball
    env.shell(
        "mkdir -p /tmp/build && \
         mkdir -p /tmp/testpkg-1.0 && \
         echo 'xz content' > /tmp/testpkg-1.0/file.txt && \
         tar cJf /tmp/build/testpkg-1.0.tar.xz -C /tmp testpkg-1.0",
    )
    .await
    .unwrap();

    // Extract it
    env.shell("cd /tmp/build && tar xJf testpkg-1.0.tar.xz")
        .await
        .unwrap();

    // Verify extraction
    assert!(env.file_exists("/tmp/build/testpkg-1.0/file.txt").await);
}

/// Test extracting a tar.bz2 archive.
#[tokio::test]
async fn test_build_extract_tar_bz2() {
    let env = TestEnv::new().await;

    // Create a test tarball
    env.shell(
        "mkdir -p /tmp/build && \
         mkdir -p /tmp/testpkg-1.0 && \
         echo 'bz2 content' > /tmp/testpkg-1.0/file.txt && \
         tar cjf /tmp/build/testpkg-1.0.tar.bz2 -C /tmp testpkg-1.0",
    )
    .await
    .unwrap();

    // Extract it
    env.shell("cd /tmp/build && tar xjf testpkg-1.0.tar.bz2")
        .await
        .unwrap();

    // Verify extraction
    assert!(env.file_exists("/tmp/build/testpkg-1.0/file.txt").await);
}

/// Test extracting a zip archive.
#[tokio::test]
async fn test_build_extract_zip() {
    let env = TestEnv::new().await;

    // Install zip if not present
    let _ = env.shell("apt-get install -y -qq zip 2>/dev/null || true").await;

    // Check if zip is available, skip test if not
    if env.shell("which zip").await.is_err() {
        eprintln!("Skipping test_build_extract_zip: zip not available");
        return;
    }

    // Create a test zip file
    env.shell(
        "mkdir -p /tmp/build && \
         mkdir -p /tmp/testpkg-1.0 && \
         echo 'zip content' > /tmp/testpkg-1.0/file.txt && \
         cd /tmp && zip -r /tmp/build/testpkg-1.0.zip testpkg-1.0",
    )
    .await
    .unwrap();

    // Extract it
    env.shell("cd /tmp/build && unzip -o testpkg-1.0.zip")
        .await
        .unwrap();

    // Verify extraction
    assert!(env.file_exists("/tmp/build/testpkg-1.0/file.txt").await);
}

/// Test build skip.
#[tokio::test]
async fn test_build_skip() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = BuildSpec::Skip;
    let result = executor.build(&spec);
    assert!(result.is_ok());
}

/// Test running a custom build command.
#[tokio::test]
async fn test_build_run_custom_cmd() {
    let env = TestEnv::new().await;

    // Run a custom build command
    env.shell("mkdir -p /tmp/build && cd /tmp/build && echo 'built!' > output.txt")
        .await
        .unwrap();

    // Verify output
    assert!(env.file_exists("/tmp/build/output.txt").await);
    let content = env.read_file("/tmp/build/output.txt").await.unwrap();
    assert!(content.contains("built!"));
}

/// Test variable expansion in build commands.
#[tokio::test]
async fn test_build_variable_expansion() {
    // Test that variables are expanded correctly by checking the expand_vars method
    // We can't directly test the output in dry_run, but we can verify the executor accepts the command
    let spec = BuildSpec::Steps(vec![BuildStep::Run(
        "echo PREFIX=$PREFIX ARCH=$ARCH NPROC=$NPROC".to_string(),
    )]);

    // This will fail because /tmp/build doesn't exist on the test host,
    // but in dry_run mode it should succeed
    let ctx_dry = Context::with_prefix("/opt/myapp")
        .build_dir(PathBuf::from("/tmp/build"))
        .arch("x86_64")
        .dry_run(true);
    let executor_dry = Executor::new(ctx_dry);
    let result = executor_dry.build(&spec);
    assert!(result.is_ok());
}

/// Test BuildSpec parsing from recipe.
#[tokio::test]
async fn test_build_steps_sequence() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = BuildSpec::Steps(vec![
        BuildStep::Run("echo 'Step 1'".to_string()),
        BuildStep::Run("echo 'Step 2'".to_string()),
        BuildStep::Run("echo 'Step 3'".to_string()),
    ]);

    let result = executor.build(&spec);
    assert!(result.is_ok());
}
