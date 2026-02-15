use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub enum Distro {
    Leviso,
    AcornOS,
    IuppiterOS,
    RalphOS,
}

impl Distro {
    pub fn dir(self) -> &'static str {
        match self {
            Self::Leviso => "leviso",
            Self::AcornOS => "AcornOS",
            Self::IuppiterOS => "IuppiterOS",
            Self::RalphOS => "RalphOS",
        }
    }

    pub fn want_suffix(self) -> &'static str {
        match self {
            Self::Leviso => "-levitate",
            Self::AcornOS => "-acorn",
            Self::IuppiterOS => "-iuppiter",
            Self::RalphOS => "-ralph",
        }
    }
}

pub fn run(distro: Option<Distro>) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let distros: Vec<Distro> = match distro {
        Some(d) => vec![d],
        None => vec![
            Distro::Leviso,
            Distro::AcornOS,
            Distro::IuppiterOS,
            Distro::RalphOS,
        ],
    };

    let mut fail = false;
    for d in distros {
        if !verify_one(&root, d) {
            fail = true;
        }
    }

    if fail {
        // Match the shell script behavior: exit non-zero if anything is missing/invalid.
        std::process::exit(1);
    }
    Ok(())
}

fn verify_one(root: &Path, distro: Distro) -> bool {
    let out_dir = artifacts_out_dir(root, distro.dir());
    let rel_file = out_dir.join("kernel-build/include/config/kernel.release");
    let vmlinuz = out_dir.join("staging/boot/vmlinuz");

    if !rel_file.is_file() {
        eprintln!("[missing] {}: {}", distro.dir(), rel_file.display());
        return false;
    }
    if !vmlinuz.is_file() {
        eprintln!("[missing] {}: {}", distro.dir(), vmlinuz.display());
        return false;
    }

    let rel = match fs::read_to_string(&rel_file) {
        Ok(s) => s.trim_end_matches(['\n', '\r']).to_string(),
        Err(_) => {
            eprintln!("[missing] {}: {}", distro.dir(), rel_file.display());
            return false;
        }
    };

    if !rel.ends_with(distro.want_suffix()) {
        eprintln!(
            "[bad] {}: kernel.release '{}' does not end with '{}'",
            distro.dir(),
            rel,
            distro.want_suffix()
        );
        return false;
    }

    let m1 = out_dir.join(format!("staging/lib/modules/{rel}"));
    let m2 = out_dir.join(format!("staging/usr/lib/modules/{rel}"));
    if !m1.is_dir() && !m2.is_dir() {
        eprintln!(
            "[missing] {}: modules dir for '{}' under staging/{{lib,usr/lib}}/modules/",
            distro.dir(),
            rel
        );
        return false;
    }

    eprintln!("[ok] {}: {}", distro.dir(), rel);
    true
}

fn artifacts_out_dir(root: &Path, distro_dir: &str) -> PathBuf {
    root.join(".artifacts/out").join(distro_dir)
}
