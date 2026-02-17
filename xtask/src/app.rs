use anyhow::Result;

pub fn run(cli: crate::cli::Cli) -> Result<()> {
    match cli.cmd {
        crate::cli::Cmd::Env { shell } => crate::tasks::tooling::env::run(shell),
        crate::cli::Cmd::Doctor => crate::tasks::tooling::doctor::run(),
        crate::cli::Cmd::Kernels { cmd } => match cmd {
            crate::cli::KernelsCmd::Build {
                distro,
                rebuild,
                autofix,
                autofix_attempts,
                autofix_prompt_file,
                llm_profile,
            } => crate::tasks::kernels::build::run(
                distro,
                rebuild,
                crate::tasks::kernels::common::AutoFixOptions {
                    enabled: autofix,
                    attempts: autofix_attempts,
                    prompt_file: autofix_prompt_file,
                    llm_profile,
                },
            ),
            crate::cli::KernelsCmd::BuildAll {
                rebuild,
                autofix,
                autofix_attempts,
                autofix_prompt_file,
                llm_profile,
            } => crate::tasks::kernels::build_all::run(
                rebuild,
                crate::tasks::kernels::common::AutoFixOptions {
                    enabled: autofix,
                    attempts: autofix_attempts,
                    prompt_file: autofix_prompt_file,
                    llm_profile,
                },
            ),
            crate::cli::KernelsCmd::Check { distro } => crate::tasks::kernels::check::run(distro),
        },
        crate::cli::Cmd::Hooks { cmd } => match cmd {
            crate::cli::HooksCmd::Install => crate::tasks::tooling::hooks::install(),
            crate::cli::HooksCmd::Remove => crate::tasks::tooling::hooks::remove(),
        },
        crate::cli::Cmd::Stages { cmd } => match cmd {
            crate::cli::StagesCmd::Boot { n, distro } => {
                crate::tasks::testing::stages::boot(n, distro)
            }
            crate::cli::StagesCmd::Test { n, distro } => {
                crate::tasks::testing::stages::test(n, distro)
            }
            crate::cli::StagesCmd::TestUpTo { n, distro } => {
                crate::tasks::testing::stages::test_up_to(n, distro)
            }
            crate::cli::StagesCmd::Status { distro } => {
                crate::tasks::testing::stages::status(distro)
            }
            crate::cli::StagesCmd::Reset { distro } => crate::tasks::testing::stages::reset(distro),
        },
    }
}
