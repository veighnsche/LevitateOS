use anyhow::Result;
use clap::{Parser, Subcommand};
use distro_builder::artifact_store::ArtifactStore;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "recart")]
#[command(about = "LevitateOS centralized artifact store manager")]
struct Cli {
    /// Repo root (auto-detected by default)
    #[arg(long)]
    repo: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show store status (counts + size)
    Status,
    /// List index entries for a kind
    Ls {
        /// Kind (e.g. rootfs_erofs, initramfs, kernel_payload)
        kind: String,
    },
    /// Garbage-collect unreferenced blobs
    Gc,
    /// Prune index entries, keeping only newest N per kind, then GC
    Prune {
        /// Keep only newest N entries per kind
        #[arg(long, default_value = "3")]
        keep_last: usize,
    },

    /// Ingest existing distro build artifacts into the centralized store (no builds).
    ///
    /// This will only ingest artifacts that already exist on disk.
    Ingest,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let repo_root = match cli.repo {
        Some(p) => p,
        None => find_repo_root(std::env::current_dir()?)?,
    };

    let store = ArtifactStore::open(&repo_root)?;

    match cli.cmd {
        Command::Status => {
            let st = store.status()?;
            println!("Artifact store: {}", st.root.display());
            println!("  Index entries:      {}", st.index_entries);
            println!("  Referenced blobs:   {}", st.referenced_blobs);
            println!("  Referenced size:    {}", fmt_bytes(st.referenced_bytes));
        }
        Command::Ls { kind } => {
            let entries = store.list_kind(&kind)?;
            if entries.is_empty() {
                println!("No entries for kind '{}'", kind);
            } else {
                for e in entries {
                    let blob = &e.blob_sha256;
                    println!(
                        "{}  key={}  blob={}  size={}",
                        e.stored_at_unix,
                        e.input_key,
                        &blob[..16.min(blob.len())],
                        fmt_bytes(e.size_bytes)
                    );
                }
            }
        }
        Command::Gc => {
            let removed = store.gc()?;
            println!("Removed {} unreferenced blob(s).", removed);
        }
        Command::Prune { keep_last } => {
            let removed_idx = store.prune_keep_last(keep_last)?;
            let removed_blobs = store.gc()?;
            println!("Removed {} index entry(s).", removed_idx);
            println!("Removed {} unreferenced blob(s).", removed_blobs);
        }
        Command::Ingest => {
            ingest_all(&repo_root, &store)?;
        }
    }

    Ok(())
}

fn ingest_all(repo_root: &Path, store: &ArtifactStore) -> Result<()> {
    let mut any = false;

    let leviso = repo_root.join("leviso");
    if leviso.exists() {
        any = true;
        ingest_leviso(&leviso, store)?;
    }

    let acorn = repo_root.join("AcornOS");
    if acorn.exists() {
        any = true;
        ingest_acorn(&acorn, store)?;
    }

    let iuppiter = repo_root.join("IuppiterOS");
    if iuppiter.exists() {
        any = true;
        ingest_iuppiter(&iuppiter, store)?;
    }

    if !any {
        anyhow::bail!(
            "No distro directories found at repo root (expected leviso/, AcornOS/, IuppiterOS/)"
        );
    }

    Ok(())
}

fn ingest_leviso(base_dir: &Path, store: &ArtifactStore) -> Result<()> {
    println!("== Ingest leviso ==");
    let out = base_dir.join("output");
    if !out.exists() {
        println!("  [SKIP] No output dir at {}", out.display());
        return Ok(());
    }

    // Ensure hash keys exist (no builds).
    leviso::rebuild::cache_kernel_hash(base_dir);
    leviso::rebuild::cache_rootfs_hash(base_dir);
    leviso::rebuild::cache_initramfs_hash(base_dir);
    leviso::rebuild::cache_install_initramfs_hash(base_dir);

    // Kernel payload (vmlinuz + modules)
    let staging = out.join("staging");
    let kernel_key = out.join(".kernel-inputs.hash");
    if staging.join("boot/vmlinuz").exists() {
        match distro_builder::artifact_store::read_input_key_file(&kernel_key)? {
            Some(key) => {
                if store.get("kernel_payload", &key)?.is_some() {
                    println!("  [SKIP] kernel_payload (already stored)");
                } else {
                    match store.put_kernel_payload(
                        &key,
                        &staging,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  kernel_payload  stored blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] kernel_payload ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] kernel_payload (missing key {})",
                kernel_key.display()
            ),
        }
    } else {
        println!("  [SKIP] kernel_payload (missing vmlinuz)");
    }

    // Rootfs
    let rootfs = out.join(distro_spec::levitate::ROOTFS_NAME);
    let rootfs_key = out.join(".rootfs-inputs.hash");
    if rootfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&rootfs_key)? {
            Some(key) => {
                if store.get("rootfs_erofs", &key)?.is_some() {
                    println!("  [SKIP] rootfs_erofs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "rootfs_erofs",
                        &key,
                        &rootfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  rootfs_erofs    ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] rootfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] rootfs_erofs (missing key {})",
                rootfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] rootfs_erofs (missing filesystem.erofs)");
    }

    // Initramfs (live)
    let initramfs = out.join(distro_spec::levitate::INITRAMFS_LIVE_OUTPUT);
    let initramfs_key = out.join(".initramfs-inputs.hash");
    if initramfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&initramfs_key)? {
            Some(key) => {
                if store.get("initramfs", &key)?.is_some() {
                    println!("  [SKIP] initramfs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "initramfs",
                        &key,
                        &initramfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  initramfs       ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] initramfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] initramfs (missing key {})",
                initramfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] initramfs (missing initramfs-live.cpio.gz)");
    }

    // Install initramfs
    let install_initramfs = out.join(distro_spec::levitate::INITRAMFS_INSTALLED_OUTPUT);
    let install_key = out.join(".install-initramfs-inputs.hash");
    if install_initramfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&install_key)? {
            Some(key) => {
                if store.get("install_initramfs", &key)?.is_some() {
                    println!("  [SKIP] install_initramfs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "install_initramfs",
                        &key,
                        &install_initramfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  install_initramfs ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] install initramfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] install_initramfs (missing key {})",
                install_key.display()
            ),
        }
    } else {
        println!("  [SKIP] install_initramfs (missing initramfs-installed.img)");
    }

    // ISO (+ checksum, if present)
    let iso = out.join(distro_spec::levitate::ISO_FILENAME);
    if iso.exists() {
        let iso_key = iso_input_key(&[
            out.join(".kernel-inputs.hash"),
            out.join(".rootfs-inputs.hash"),
            out.join(".initramfs-inputs.hash"),
            out.join(".install-initramfs-inputs.hash"),
        ]);

        match iso_key {
            Some(key) => {
                if store.get("iso", &key)?.is_some() {
                    println!("  [SKIP] iso (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "iso",
                        &key,
                        &iso,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  iso            ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] iso ingest failed: {:#}", e),
                    }
                }

                if let Some(checksum) = find_iso_checksum_file(&iso) {
                    if checksum.exists() {
                        if store.get("iso_checksum", &key)?.is_some() {
                            println!("  [SKIP] iso_checksum (already stored)");
                        } else {
                            match store.ingest_file_move_and_link(
                                "iso_checksum",
                                &key,
                                &checksum,
                                std::collections::BTreeMap::new(),
                            ) {
                                Ok(sha) => {
                                    println!("  iso_checksum    ingested blob={}", &sha[..16])
                                }
                                Err(e) => eprintln!("  [WARN] iso checksum ingest failed: {:#}", e),
                            }
                        }
                    }
                } else {
                    println!("  [SKIP] iso_checksum (not found)");
                }
            }
            None => println!("  [SKIP] iso (missing inputs-hash key)"),
        }
    } else {
        println!("  [SKIP] iso (missing ISO file)");
    }

    Ok(())
}

fn ingest_acorn(base_dir: &Path, store: &ArtifactStore) -> Result<()> {
    println!("== Ingest AcornOS ==");
    let out = base_dir.join("output");
    if !out.exists() {
        println!("  [SKIP] No output dir at {}", out.display());
        return Ok(());
    }

    // Ensure hash keys exist (no builds).
    acornos::rebuild::cache_kernel_hash(base_dir);
    acornos::rebuild::cache_rootfs_hash(base_dir);
    acornos::rebuild::cache_initramfs_hash(base_dir);

    // Kernel payload
    let staging = out.join("staging");
    let kernel_key = out.join(".kernel-inputs.hash");
    if staging.join("boot/vmlinuz").exists() {
        match distro_builder::artifact_store::read_input_key_file(&kernel_key)? {
            Some(key) => {
                if store.get("kernel_payload", &key)?.is_some() {
                    println!("  [SKIP] kernel_payload (already stored)");
                } else {
                    match store.put_kernel_payload(
                        &key,
                        &staging,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  kernel_payload  stored blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] kernel_payload ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] kernel_payload (missing key {})",
                kernel_key.display()
            ),
        }
    } else {
        println!("  [SKIP] kernel_payload (missing vmlinuz)");
    }

    // Rootfs
    let rootfs = out.join(distro_spec::acorn::ROOTFS_NAME);
    let rootfs_key = out.join(".rootfs-inputs.hash");
    if rootfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&rootfs_key)? {
            Some(key) => {
                if store.get("rootfs_erofs", &key)?.is_some() {
                    println!("  [SKIP] rootfs_erofs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "rootfs_erofs",
                        &key,
                        &rootfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  rootfs_erofs    ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] rootfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] rootfs_erofs (missing key {})",
                rootfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] rootfs_erofs (missing filesystem.erofs)");
    }

    // Initramfs (live)
    let initramfs = out.join(distro_spec::acorn::INITRAMFS_LIVE_OUTPUT);
    let initramfs_key = out.join(".initramfs-inputs.hash");
    if initramfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&initramfs_key)? {
            Some(key) => {
                if store.get("initramfs", &key)?.is_some() {
                    println!("  [SKIP] initramfs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "initramfs",
                        &key,
                        &initramfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  initramfs       ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] initramfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] initramfs (missing key {})",
                initramfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] initramfs (missing initramfs-live.cpio.gz)");
    }

    // ISO (+ checksum, if present)
    let iso = out.join(distro_spec::acorn::ISO_FILENAME);
    if iso.exists() {
        let iso_key = iso_input_key(&[
            out.join(".kernel-inputs.hash"),
            out.join(".rootfs-inputs.hash"),
            out.join(".initramfs-inputs.hash"),
        ]);

        match iso_key {
            Some(key) => {
                if store.get("iso", &key)?.is_some() {
                    println!("  [SKIP] iso (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "iso",
                        &key,
                        &iso,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  iso            ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] iso ingest failed: {:#}", e),
                    }
                }

                if let Some(checksum) = find_iso_checksum_file(&iso) {
                    if checksum.exists() {
                        if store.get("iso_checksum", &key)?.is_some() {
                            println!("  [SKIP] iso_checksum (already stored)");
                        } else {
                            match store.ingest_file_move_and_link(
                                "iso_checksum",
                                &key,
                                &checksum,
                                std::collections::BTreeMap::new(),
                            ) {
                                Ok(sha) => {
                                    println!("  iso_checksum    ingested blob={}", &sha[..16])
                                }
                                Err(e) => eprintln!("  [WARN] iso checksum ingest failed: {:#}", e),
                            }
                        }
                    }
                } else {
                    println!("  [SKIP] iso_checksum (not found)");
                }
            }
            None => println!("  [SKIP] iso (missing inputs-hash key)"),
        }
    } else {
        println!("  [SKIP] iso (missing ISO file)");
    }

    Ok(())
}

fn ingest_iuppiter(base_dir: &Path, store: &ArtifactStore) -> Result<()> {
    println!("== Ingest IuppiterOS ==");
    let out = base_dir.join("output");
    if !out.exists() {
        println!("  [SKIP] No output dir at {}", out.display());
        return Ok(());
    }

    // Ensure hash keys exist (no builds).
    iuppiteros::rebuild::cache_kernel_hash(base_dir);
    iuppiteros::rebuild::cache_rootfs_hash(base_dir);
    iuppiteros::rebuild::cache_initramfs_hash(base_dir);

    // Kernel payload
    let staging = out.join("staging");
    let kernel_key = out.join(".kernel-inputs.hash");
    if staging.join("boot/vmlinuz").exists() {
        match distro_builder::artifact_store::read_input_key_file(&kernel_key)? {
            Some(key) => {
                if store.get("kernel_payload", &key)?.is_some() {
                    println!("  [SKIP] kernel_payload (already stored)");
                } else {
                    match store.put_kernel_payload(
                        &key,
                        &staging,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  kernel_payload  stored blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] kernel_payload ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] kernel_payload (missing key {})",
                kernel_key.display()
            ),
        }
    } else {
        println!("  [SKIP] kernel_payload (missing vmlinuz)");
    }

    // Rootfs
    let rootfs = out.join(distro_spec::iuppiter::ROOTFS_NAME);
    let rootfs_key = out.join(".rootfs-inputs.hash");
    if rootfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&rootfs_key)? {
            Some(key) => {
                if store.get("rootfs_erofs", &key)?.is_some() {
                    println!("  [SKIP] rootfs_erofs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "rootfs_erofs",
                        &key,
                        &rootfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  rootfs_erofs    ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] rootfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] rootfs_erofs (missing key {})",
                rootfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] rootfs_erofs (missing filesystem.erofs)");
    }

    // Initramfs (live)
    let initramfs = out.join(distro_spec::iuppiter::INITRAMFS_LIVE_OUTPUT);
    let initramfs_key = out.join(".initramfs-inputs.hash");
    if initramfs.exists() {
        match distro_builder::artifact_store::read_input_key_file(&initramfs_key)? {
            Some(key) => {
                if store.get("initramfs", &key)?.is_some() {
                    println!("  [SKIP] initramfs (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "initramfs",
                        &key,
                        &initramfs,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  initramfs       ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] initramfs ingest failed: {:#}", e),
                    }
                }
            }
            None => println!(
                "  [SKIP] initramfs (missing key {})",
                initramfs_key.display()
            ),
        }
    } else {
        println!("  [SKIP] initramfs (missing initramfs-live.cpio.gz)");
    }

    // ISO (+ checksum, if present)
    let iso = out.join(distro_spec::iuppiter::ISO_FILENAME);
    if iso.exists() {
        let iso_key = iso_input_key(&[
            out.join(".kernel-inputs.hash"),
            out.join(".rootfs-inputs.hash"),
            out.join(".initramfs-inputs.hash"),
        ]);

        match iso_key {
            Some(key) => {
                if store.get("iso", &key)?.is_some() {
                    println!("  [SKIP] iso (already stored)");
                } else {
                    match store.ingest_file_move_and_link(
                        "iso",
                        &key,
                        &iso,
                        std::collections::BTreeMap::new(),
                    ) {
                        Ok(sha) => println!("  iso            ingested blob={}", &sha[..16]),
                        Err(e) => eprintln!("  [WARN] iso ingest failed: {:#}", e),
                    }
                }

                if let Some(checksum) = find_iso_checksum_file(&iso) {
                    if checksum.exists() {
                        if store.get("iso_checksum", &key)?.is_some() {
                            println!("  [SKIP] iso_checksum (already stored)");
                        } else {
                            match store.ingest_file_move_and_link(
                                "iso_checksum",
                                &key,
                                &checksum,
                                std::collections::BTreeMap::new(),
                            ) {
                                Ok(sha) => {
                                    println!("  iso_checksum    ingested blob={}", &sha[..16])
                                }
                                Err(e) => eprintln!("  [WARN] iso checksum ingest failed: {:#}", e),
                            }
                        }
                    }
                } else {
                    println!("  [SKIP] iso_checksum (not found)");
                }
            }
            None => println!("  [SKIP] iso (missing inputs-hash key)"),
        }
    } else {
        println!("  [SKIP] iso (missing ISO file)");
    }

    Ok(())
}

fn iso_input_key(inputs_hash_files: &[PathBuf]) -> Option<String> {
    let refs: Vec<&Path> = inputs_hash_files.iter().map(|p| p.as_path()).collect();
    distro_builder::cache::hash_files(&refs)
}

fn find_iso_checksum_file(iso_path: &Path) -> Option<PathBuf> {
    // Expected: replace extension (foo.iso -> foo.sha512)
    let replaced = iso_path.with_extension("sha512");
    if replaced.exists() {
        return Some(replaced);
    }

    // Older/buggy case: appended suffix (foo.iso -> foo.iso.sha512)
    let appended = PathBuf::from(format!("{}.sha512", iso_path.display()));
    if appended.exists() {
        return Some(appended);
    }

    None
}

fn find_repo_root(start: PathBuf) -> Result<PathBuf> {
    let mut cur = start;
    loop {
        if cur.join(".git").exists() && cur.join("Cargo.toml").exists() {
            return Ok(cur);
        }
        if !cur.pop() {
            break;
        }
    }
    anyhow::bail!("Could not auto-detect repo root. Use --repo /path/to/LevitateOS");
}

fn fmt_bytes(n: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let f = n as f64;
    if f >= TB {
        format!("{:.2} TB", f / TB)
    } else if f >= GB {
        format!("{:.2} GB", f / GB)
    } else if f >= MB {
        format!("{:.2} MB", f / MB)
    } else if f >= KB {
        format!("{:.2} KB", f / KB)
    } else {
        format!("{} B", n)
    }
}
