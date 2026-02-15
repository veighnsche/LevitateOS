use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use std::process::Command;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "LevitateOS repo task runner (wraps the justfile)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run `just` with raw arguments (pass-through).
    ///
    /// Example: cargo xtask just -- --list
    Just {
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Build an ISO (delegates to `just build`).
    Build {
        #[arg(default_value = "leviso")]
        distro: String,
    },

    /// Run an interactive checkpoint (delegates to `just checkpoint`).
    Checkpoint {
        n: u8,
        #[arg(default_value = "leviso")]
        distro: String,
    },

    /// Run automated checkpoint N (delegates to `just test`).
    Test {
        n: u8,
        #[arg(default_value = "levitate")]
        distro: String,
    },

    /// Run all automated checkpoints up to N (delegates to `just test-up-to`).
    TestUpTo {
        n: u8,
        #[arg(default_value = "levitate")]
        distro: String,
    },

    /// Show checkpoint status (delegates to `just test-status`).
    TestStatus {
        #[arg(default_value = "levitate")]
        distro: String,
    },

    /// Reset checkpoint state (delegates to `just test-reset`).
    TestReset {
        #[arg(default_value = "levitate")]
        distro: String,
    },

    /// Docs tasks (delegates to justfile recipes).
    #[command(subcommand)]
    Docs(DocsCmd),
}

#[derive(Subcommand)]
enum DocsCmd {
    ContentBuild,
    ContentCheck,
    WebsiteDev,
    WebsiteBuild,
    WebsiteTypecheck,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.cmd {
        Cmd::Just { args } => run_just(args),
        Cmd::Build { distro } => run_just(vec!["build".into(), distro]),
        Cmd::Checkpoint { n, distro } => run_just(vec!["checkpoint".into(), n.to_string(), distro]),
        Cmd::Test { n, distro } => run_just(vec!["test".into(), n.to_string(), distro]),
        Cmd::TestUpTo { n, distro } => run_just(vec!["test-up-to".into(), n.to_string(), distro]),
        Cmd::TestStatus { distro } => run_just(vec!["test-status".into(), distro]),
        Cmd::TestReset { distro } => run_just(vec!["test-reset".into(), distro]),
        Cmd::Docs(cmd) => match cmd {
            DocsCmd::ContentBuild => run_just(vec!["docs-content-build".into()]),
            DocsCmd::ContentCheck => run_just(vec!["docs-content-check".into()]),
            DocsCmd::WebsiteDev => run_just(vec!["website-dev".into()]),
            DocsCmd::WebsiteBuild => run_just(vec!["website-build".into()]),
            DocsCmd::WebsiteTypecheck => run_just(vec!["website-typecheck".into()]),
        },
    }
}

fn run_just(args: Vec<String>) -> Result<()> {
    which::which("just").context("`just` not found in PATH")?;

    // Ensure we're running from repo root so relative paths in justfile work.
    let repo_root = std::env::current_dir().context("failed to get current dir")?;

    let status = Command::new("just")
        .current_dir(&repo_root)
        .args(args)
        .status()
        .context("failed to spawn just")?;

    if !status.success() {
        bail!("just failed with exit code {:?}", status.code());
    }
    Ok(())
}
