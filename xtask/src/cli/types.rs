use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Distro {
    #[value(name = "leviso")]
    Leviso,
    #[value(name = "acorn")]
    AcornOS,
    #[value(name = "iuppiter")]
    IuppiterOS,
    #[value(name = "ralph")]
    RalphOS,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BootDistro {
    #[value(name = "levitate")]
    Levitate,

    #[value(name = "acorn")]
    Acorn,

    #[value(name = "iuppiter")]
    Iuppiter,

    #[value(name = "ralph")]
    Ralph,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HarnessDistro {
    #[value(name = "levitate")]
    Levitate,

    #[value(name = "acorn")]
    Acorn,

    #[value(name = "iuppiter")]
    Iuppiter,

    #[value(name = "ralph")]
    Ralph,
}

impl HarnessDistro {
    pub fn id(self) -> &'static str {
        match self {
            Self::Levitate => "levitate",
            Self::Acorn => "acorn",
            Self::Iuppiter => "iuppiter",
            Self::Ralph => "ralph",
        }
    }
}

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "LevitateOS repo developer tasks (scaffolding; complements justfile)")]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Print the environment exports that the justfile sets for QEMU/tooling.
    ///
    /// Usage:
    ///   eval "$(cargo xtask env bash)"
    Env {
        #[arg(value_enum, default_value_t = Shell::Bash)]
        shell: Shell,
    },

    /// Check that the local toolchain/tools match what the justfile expects.
    Doctor,

    /// Kernel-related tasks.
    Kernels {
        #[command(subcommand)]
        cmd: KernelsCmd,
    },

    /// Install/remove shared git hooks (pre-commit) across the workspace + Rust submodules.
    Hooks {
        #[command(subcommand)]
        cmd: HooksCmd,
    },

    /// Install-test stage runner (interactive boot + automated pass/fail checks).
    Stages {
        #[command(subcommand)]
        cmd: StagesCmd,
    },

    /// Repository policy checks.
    Policy {
        #[command(subcommand)]
        cmd: PolicyCmd,
    },
}

#[derive(Subcommand)]
pub enum KernelsCmd {
    /// Build the kernel for one distro (policy window enforced).
    #[command(name = "build")]
    Build {
        #[arg(value_enum)]
        distro: Distro,

        #[arg(
            long = "rebuild",
            help = "Force the selected distro to rebuild+reinstall its kernel even if artifacts are already present. Does not bypass the 23:00-10:00 build-hours policy."
        )]
        rebuild: bool,

        #[arg(
            long = "autofix",
            help = "On kernel build failure, rerun via `recipe install --autofix ...` so Recipe can ask the configured LLM provider to propose a patch and retry."
        )]
        autofix: bool,

        #[arg(
            long = "autofix-attempts",
            default_value_t = 2,
            help = "Maximum number of auto-fix attempts per failing kernel build."
        )]
        autofix_attempts: u8,

        #[arg(
            long = "autofix-prompt-file",
            help = "Optional extra instructions appended to the built-in Codex autofix prompt."
        )]
        autofix_prompt_file: Option<PathBuf>,

        #[arg(
            long = "llm-profile",
            help = "Pass through to `recipe --llm-profile <name>` (selects a profile from XDG `recipe/llm.toml`)."
        )]
        llm_profile: Option<String>,
    },

    /// Build kernels for all distros (policy window enforced).
    #[command(name = "build-all")]
    BuildAll {
        #[arg(
            long = "rebuild",
            help = "Force every distro to rebuild+reinstall its kernel even if artifacts are already present. Does not bypass the 23:00-10:00 build-hours policy."
        )]
        rebuild: bool,

        #[arg(
            long = "autofix",
            help = "On kernel build failure, rerun via `recipe install --autofix ...` so Recipe can ask the configured LLM provider to propose a patch and retry."
        )]
        autofix: bool,

        #[arg(
            long = "autofix-attempts",
            default_value_t = 2,
            help = "Maximum number of auto-fix attempts per failing kernel build."
        )]
        autofix_attempts: u8,

        #[arg(
            long = "autofix-prompt-file",
            help = "Optional extra instructions appended to the built-in Codex autofix prompt."
        )]
        autofix_prompt_file: Option<PathBuf>,

        #[arg(
            long = "llm-profile",
            help = "Pass through to `recipe --llm-profile <name>` (selects a profile from XDG `recipe/llm.toml`)."
        )]
        llm_profile: Option<String>,
    },

    /// Verify built kernel artifacts for one distro (or all distros if omitted).
    Check {
        #[arg(value_enum)]
        distro: Option<Distro>,
    },
}

#[derive(Subcommand)]
pub enum HooksCmd {
    /// Install the shared pre-commit hook into the workspace + Rust submodules.
    Install,

    /// Remove the shared pre-commit hook from the workspace + Rust submodules.
    Remove,
}

#[derive(Subcommand)]
pub enum StagesCmd {
    /// Boot into an interactive stage (serial console).
    ///
    /// Interactive stages: 01 (live ISO), 02 (live tools), 04 (installed).
    Boot {
        n: u8,
        #[arg(value_enum, default_value_t = BootDistro::Levitate)]
        distro: BootDistro,
        #[arg(long, value_name = "KEY=VALUE[,KEY=VALUE...]")]
        inject: Option<String>,
        #[arg(long, value_name = "PATH")]
        inject_file: Option<PathBuf>,
        /// Boot the stage and wait for SSH readiness on the host forwarded port.
        #[arg(long)]
        ssh: bool,
        /// SSH host-forward port when `--ssh` is enabled.
        #[arg(long, default_value_t = 2222)]
        ssh_port: u16,
        /// Timeout in seconds to wait for SSH readiness and probe when `--ssh` is enabled.
        #[arg(long, default_value_t = 90)]
        ssh_timeout: u64,
        /// Connect and verify SSH only, without opening an interactive shell.
        #[arg(long)]
        no_shell: bool,
        /// SSH private key used for interactive or probe login when `--ssh` is enabled.
        #[arg(long, value_name = "PATH")]
        ssh_private_key: Option<PathBuf>,
    },

    /// Run automated stage test N (pass/fail).
    Test {
        n: u8,
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
        #[arg(long, value_name = "KEY=VALUE[,KEY=VALUE...]")]
        inject: Option<String>,
        #[arg(long, value_name = "PATH")]
        inject_file: Option<PathBuf>,
    },

    /// Run all automated stage tests up to N.
    TestUpTo {
        n: u8,
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
        #[arg(long, value_name = "KEY=VALUE[,KEY=VALUE...]")]
        inject: Option<String>,
        #[arg(long, value_name = "PATH")]
        inject_file: Option<PathBuf>,
    },

    /// Show stage test status.
    Status {
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },

    /// Reset cached stage state for a distro.
    Reset {
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },
}

#[derive(Subcommand)]
pub enum PolicyCmd {
    /// Fail if forbidden legacy bindings appear in code/config for stage wiring.
    #[command(name = "audit-legacy-bindings")]
    AuditLegacyBindings,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Sh,
}
