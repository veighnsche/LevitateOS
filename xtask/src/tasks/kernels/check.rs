use anyhow::Result;

pub fn run(distro: Option<crate::cli::Distro>) -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let distros: Vec<crate::cli::Distro> = match distro {
        Some(d) => vec![d],
        None => vec![
            crate::cli::Distro::Leviso,
            crate::cli::Distro::AcornOS,
            crate::cli::Distro::IuppiterOS,
            crate::cli::Distro::RalphOS,
        ],
    };

    let mut fail = false;
    for d in distros {
        let t = super::common::target_for(d);
        match super::common::verify_one(&root, &t) {
            Ok(rel) => eprintln!("[ok] {}: {}", t.distro_id, rel),
            Err(e) => {
                fail = true;
                eprintln!("[bad] {}: {:#}", t.distro_id, e);
            }
        }
    }

    if fail {
        // Match the legacy shell wrapper behavior: exit non-zero if anything is missing/invalid.
        std::process::exit(1);
    }
    Ok(())
}
