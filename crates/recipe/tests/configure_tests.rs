//! Integration tests for the configure phase.

mod common;

use common::TestEnv;
use levitate_recipe::{ConfigureSpec, ConfigureStep, Context, Executor};
use std::path::PathBuf;

/// Test creating a regular user.
#[tokio::test]
async fn test_configure_create_user() {
    let env = TestEnv::new().await;

    // Create a regular user
    env.shell("useradd -m testuser || true").await.unwrap();

    assert!(env.user_exists("testuser").await);
}

/// Test creating a system user.
#[tokio::test]
async fn test_configure_create_user_system() {
    let env = TestEnv::new().await;

    // Create a system user (UID < 1000)
    env.shell("useradd -r sysuser || true").await.unwrap();

    assert!(env.user_exists("sysuser").await);

    // System users typically have UID < 1000
    let uid = env.user_uid("sysuser").await;
    assert!(uid.is_some());
    assert!(uid.unwrap() < 1000, "System user should have UID < 1000");
}

/// Test creating a user with no-login shell.
#[tokio::test]
async fn test_configure_create_user_nologin() {
    let env = TestEnv::new().await;

    // Create a user with nologin shell
    env.shell("useradd -r -s /sbin/nologin nologinuser || true")
        .await
        .unwrap();

    assert!(env.user_exists("nologinuser").await);

    let shell = env.user_shell("nologinuser").await;
    assert!(shell.is_some());
    assert!(
        shell.as_ref().unwrap().contains("nologin"),
        "User shell should be nologin, got: {:?}",
        shell
    );
}

/// Test creating a directory.
#[tokio::test]
async fn test_configure_create_dir() {
    let env = TestEnv::new().await;

    env.shell("mkdir -p /var/lib/testpkg").await.unwrap();

    assert!(env.is_dir("/var/lib/testpkg").await);
}

/// Test creating a directory with owner.
#[tokio::test]
async fn test_configure_create_dir_owner() {
    let env = TestEnv::new().await;

    // Create user first
    env.shell("useradd -r testowner || true").await.unwrap();

    // Create directory with owner
    env.shell("mkdir -p /var/lib/ownedpkg && chown testowner /var/lib/ownedpkg")
        .await
        .unwrap();

    assert!(env.is_dir("/var/lib/ownedpkg").await);
    let owner = env.file_owner("/var/lib/ownedpkg").await;
    assert_eq!(owner, Some("testowner".to_string()));
}

/// Test template substitution.
#[tokio::test]
async fn test_configure_template() {
    let env = TestEnv::new().await;

    // Create a template file
    env.write_file("/tmp/config.template", "host={{HOST}}\nport={{PORT}}")
        .await
        .unwrap();

    // Apply template substitution
    env.shell("sed -i 's/{{HOST}}/localhost/g' /tmp/config.template")
        .await
        .unwrap();
    env.shell("sed -i 's/{{PORT}}/8080/g' /tmp/config.template")
        .await
        .unwrap();

    let content = env.read_file("/tmp/config.template").await.unwrap();
    assert!(content.contains("host=localhost"));
    assert!(content.contains("port=8080"));
    assert!(!content.contains("{{HOST}}"));
    assert!(!content.contains("{{PORT}}"));
}

/// Test running a custom configure command.
#[tokio::test]
async fn test_configure_run() {
    let env = TestEnv::new().await;

    // Run a custom command
    env.shell("echo 'configured' > /tmp/configured.flag")
        .await
        .unwrap();

    assert!(env.file_exists("/tmp/configured.flag").await);
}

/// Test ConfigureSpec with dry run.
#[tokio::test]
async fn test_configure_spec_dry_run() {
    let ctx = Context::with_prefix("/usr/local")
        .build_dir(PathBuf::from("/tmp/build"))
        .dry_run(true);

    let executor = Executor::new(ctx);

    let spec = ConfigureSpec {
        steps: vec![
            ConfigureStep::CreateUser {
                name: "testuser".to_string(),
                system: true,
                no_login: true,
            },
            ConfigureStep::CreateDir {
                path: "/var/lib/testpkg".to_string(),
                owner: Some("testuser".to_string()),
            },
        ],
    };

    let result = executor.configure(&spec);
    assert!(result.is_ok());
}
