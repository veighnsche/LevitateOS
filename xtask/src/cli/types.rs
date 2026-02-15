use clap::{Parser, Subcommand, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Distro {
    #[value(name = "leviso")]
    Leviso,
    #[value(name = "AcornOS")]
    AcornOS,
    #[value(name = "IuppiterOS")]
    IuppiterOS,
    #[value(name = "RalphOS")]
    RalphOS,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum BootDistro {
    #[value(name = "leviso", alias = "levitate")]
    Leviso,

    #[value(name = "acorn", alias = "AcornOS")]
    Acorn,

    #[value(name = "iuppiter", alias = "IuppiterOS")]
    Iuppiter,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum HarnessDistro {
    #[value(name = "levitate", alias = "leviso")]
    Levitate,

    #[value(name = "acorn", alias = "AcornOS")]
    Acorn,

    #[value(name = "iuppiter", alias = "IuppiterOS")]
    Iuppiter,
}

impl HarnessDistro {
    pub fn id(self) -> &'static str {
        match self {
            Self::Levitate => "levitate",
            Self::Acorn => "acorn",
            Self::Iuppiter => "iuppiter",
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

    /// Install-test checkpoint runner (interactive boot + automated pass/fail checks).
    Checkpoints {
        #[command(subcommand)]
        cmd: CheckpointsCmd,
    },
}

#[derive(Subcommand)]
pub enum KernelsCmd {
    /// Verify built kernel artifacts for one distro (or all distros if omitted).
    Check {
        #[arg(value_enum)]
        distro: Option<Distro>,
    },

    /// Build kernels for all distros (x86_64 policy window enforced).
    BuildAllX86_64 {
        #[arg(
            long = "rebuild",
            visible_alias = "force",
            help = "Ignore the 'already built+verified' fast-path so every distro rebuilds. This purges `.artifacts/out/<distro>/{kernel-build,staging/boot/vmlinuz,staging/{lib,usr/lib}/modules}` before building. Does not bypass the 23:00-10:00 build-hours policy."
        )]
        rebuild: bool,
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
pub enum CheckpointsCmd {
    /// Boot into an interactive checkpoint stage (serial console).
    ///
    /// Interactive checkpoints: 1 (live ISO), 2 (live tools), 4 (installed).
    Boot {
        n: u8,
        #[arg(value_enum, default_value_t = BootDistro::Leviso)]
        distro: BootDistro,
    },

    /// Run automated checkpoint test N (pass/fail).
    Test {
        n: u8,
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },

    /// Run all automated checkpoint tests up to N.
    TestUpTo {
        n: u8,
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },

    /// Show checkpoint test status.
    Status {
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },

    /// Reset cached checkpoint state for a distro.
    Reset {
        #[arg(value_enum, default_value_t = HarnessDistro::Levitate)]
        distro: HarnessDistro,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Shell {
    Bash,
    Sh,
}
