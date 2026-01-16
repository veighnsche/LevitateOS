//! Integration tests for the install phase.

mod common;

use common::TestEnv;
use levitate_recipe::{Context, Executor, InstallFile, InstallSpec};
use std::path::PathBuf;

/// Test installing a binary to $PREFIX/bin.
#[tokio::test]
async fn test_install_to_bin() {
    let env = TestEnv::new().await;

    // Create a test binary in build dir
    env.shell(
        "mkdir -p /tmp/build && \
         echo '#!/bin/sh' > /tmp/build/mybin && \
         echo 'echo hello' >> /tmp/build/mybin && \
         chmod +x /tmp/build/mybin",
    )
    .await
    .unwrap();

    // Install it
    env.shell("install -Dm755 /tmp/build/mybin /usr/local/bin/mybin")
        .await
        .unwrap();

    // Verify
    assert!(env.file_exists("/usr/local/bin/mybin").await);
    let mode = env.file_mode("/usr/local/bin/mybin").await;
    assert_eq!(mode, Some(0o755));
}

/// Test installing a binary with custom mode.
#[tokio::test]
async fn test_install_to_bin_mode() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo '#!/bin/sh' > /tmp/build/mybin")
        .await
        .unwrap();

    // Install with mode 700
    env.shell("install -Dm700 /tmp/build/mybin /usr/local/bin/mybin700")
        .await
        .unwrap();

    let mode = env.file_mode("/usr/local/bin/mybin700").await;
    assert_eq!(mode, Some(0o700));
}

/// Test installing a binary with rename.
#[tokio::test]
async fn test_install_to_bin_rename() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo '#!/bin/sh' > /tmp/build/originalname")
        .await
        .unwrap();

    // Install with different name
    env.shell("install -Dm755 /tmp/build/originalname /usr/local/bin/newname")
        .await
        .unwrap();

    assert!(env.file_exists("/usr/local/bin/newname").await);
    assert!(!env.file_exists("/usr/local/bin/originalname").await);
}

/// Test installing a library to $PREFIX/lib.
#[tokio::test]
async fn test_install_to_lib() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo 'fake library' > /tmp/build/libtest.so")
        .await
        .unwrap();

    env.shell("install -Dm644 /tmp/build/libtest.so /usr/local/lib/libtest.so")
        .await
        .unwrap();

    assert!(env.file_exists("/usr/local/lib/libtest.so").await);
    let mode = env.file_mode("/usr/local/lib/libtest.so").await;
    assert_eq!(mode, Some(0o644));
}

/// Test installing a config file to absolute path.
#[tokio::test]
async fn test_install_to_config() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo 'key=value' > /tmp/build/myapp.conf")
        .await
        .unwrap();

    env.shell("install -Dm644 /tmp/build/myapp.conf /etc/myapp/config.conf")
        .await
        .unwrap();

    assert!(env.file_exists("/etc/myapp/config.conf").await);
    let content = env.read_file("/etc/myapp/config.conf").await.unwrap();
    assert!(content.contains("key=value"));
}

/// Test installing a man page.
#[tokio::test]
async fn test_install_to_man() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo '.TH MYBIN 1' > /tmp/build/mybin.1")
        .await
        .unwrap();

    // Man pages go to share/man/manN/
    env.shell("install -Dm644 /tmp/build/mybin.1 /usr/local/share/man/man1/mybin.1")
        .await
        .unwrap();

    assert!(env.file_exists("/usr/local/share/man/man1/mybin.1").await);
}

/// Test installing a data file to $PREFIX/share.
#[tokio::test]
async fn test_install_to_share() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /tmp/build && echo 'data content' > /tmp/build/data.txt")
        .await
        .unwrap();

    env.shell("install -Dm644 /tmp/build/data.txt /usr/local/share/myapp/data.txt")
        .await
        .unwrap();

    assert!(env.file_exists("/usr/local/share/myapp/data.txt").await);
}

/// Test creating a symlink.
#[tokio::test]
async fn test_install_link() {
    let env = TestEnv::new().await;

    // Create original file
    env.shell("mkdir -p /usr/local/bin && echo '#!/bin/sh' > /usr/local/bin/original")
        .await
        .unwrap();

    // Create symlink
    env.shell("ln -sf /usr/local/bin/original /usr/local/bin/alias")
        .await
        .unwrap();

    // Verify symlink exists and points to original
    let result = env.shell("readlink /usr/local/bin/alias").await.unwrap();
    assert!(result.contains("/usr/local/bin/original"));
}

/// Test InstallSpec with dry run.
#[tokio::test]
async fn test_install_spec_dry_run() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = InstallSpec {
        files: vec![
            InstallFile::ToBin {
                src: "mybin".to_string(),
                dest: None,
                mode: Some(0o755),
            },
            InstallFile::ToLib {
                src: "mylib.so".to_string(),
                dest: None,
            },
        ],
    };

    let result = executor.install(&spec);
    assert!(result.is_ok());
}
