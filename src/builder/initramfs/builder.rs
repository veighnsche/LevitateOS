//! Initramfs builder - constructs CPIO from manifest
//!
//! TEAM_474: Event-driven builder for TUI progress reporting.

use super::cpio::CpioArchive;
use super::manifest::{FileEntry, Manifest, parse_mode};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Build events emitted during construction
#[derive(Clone, Debug)]
pub enum BuildEvent {
    /// Phase started
    PhaseStart { name: &'static str, total: usize },
    /// Phase completed
    PhaseComplete { name: &'static str },
    /// Directory created
    DirectoryCreated { path: String },
    /// Binary added
    BinaryAdded { path: String, size: u64 },
    /// Symlink created
    SymlinkCreated { link: String, target: String },
    /// File added
    FileAdded { path: String, size: u64 },
    /// Device node created
    DeviceCreated { path: String },
    /// Build completed successfully
    BuildComplete {
        output_path: PathBuf,
        total_size: u64,
        duration: Duration,
    },
    /// Build failed
    BuildFailed { error: String },
}

/// Initramfs builder
pub struct InitramfsBuilder {
    manifest: Manifest,
    arch: String,
    base_dir: PathBuf,
}

impl InitramfsBuilder {
    pub fn new(manifest: Manifest, arch: &str, base_dir: &Path) -> Self {
        Self {
            manifest,
            arch: arch.to_string(),
            base_dir: base_dir.to_path_buf(),
        }
    }

    /// Build the initramfs without events (simple mode)
    pub fn build(&self) -> Result<PathBuf> {
        self.build_with_events(|_| {})
    }

    /// Build the initramfs, emitting events for progress
    pub fn build_with_events<F>(&self, emit: F) -> Result<PathBuf>
    where
        F: Fn(BuildEvent),
    {
        let start = Instant::now();
        let mut archive = CpioArchive::new();

        // 1. Create directories
        let dirs = &self.manifest.layout.directories;
        emit(BuildEvent::PhaseStart {
            name: "Creating directories",
            total: dirs.len(),
        });
        for dir in dirs {
            archive.add_directory(dir, 0o755);
            emit(BuildEvent::DirectoryCreated { path: dir.clone() });
        }
        // Also add /lib and /usr/lib for musl linker support
        archive.add_directory("lib", 0o755);
        archive.add_directory("usr", 0o755);
        archive.add_directory("usr/lib", 0o755);
        emit(BuildEvent::PhaseComplete {
            name: "Creating directories",
        });

        // 2. Add binaries
        let binaries: Vec<_> = self
            .manifest
            .binaries
            .iter()
            .filter(|(_, b)| !b.source.is_empty())
            .collect();
        emit(BuildEvent::PhaseStart {
            name: "Adding binaries",
            total: binaries.len(),
        });
        for (name, binary) in &binaries {
            let source_path = PathBuf::from(&binary.source);
            let data = std::fs::read(&source_path)
                .with_context(|| format!("Failed to read binary '{}': {}", name, source_path.display()))?;
            let size = data.len() as u64;
            let mode = parse_mode(&binary.mode);
            archive.add_file(&binary.dest, &data, mode);
            emit(BuildEvent::BinaryAdded {
                path: binary.dest.clone(),
                size,
            });

            // Copy busybox to /init for custom kernel compatibility
            // (Custom LevitateOS kernel can't follow symlinks for init)
            // Linux works with either symlink or file, so file is safe for both
            if *name == "busybox" {
                archive.add_file("init", &data, mode);
                emit(BuildEvent::BinaryAdded {
                    path: "/init".to_string(),
                    size,
                });
            }
        }

        // Note: musl dynamic linker not needed - BusyBox is statically linked

        emit(BuildEvent::PhaseComplete {
            name: "Adding binaries",
        });

        // 3. Create symlinks
        let symlinks = &self.manifest.symlinks;
        emit(BuildEvent::PhaseStart {
            name: "Creating symlinks",
            total: symlinks.len(),
        });
        for (link, target) in symlinks {
            archive.add_symlink(link, target);
            emit(BuildEvent::SymlinkCreated {
                link: link.clone(),
                target: target.clone(),
            });
        }
        // Add /usr/lib -> ../lib symlink for library compatibility
        archive.add_symlink("usr/lib", "../lib");
        emit(BuildEvent::PhaseComplete {
            name: "Creating symlinks",
        });

        // 4. Add files
        let files_total = self.manifest.files.len() + self.manifest.scripts.len();
        emit(BuildEvent::PhaseStart {
            name: "Adding files",
            total: files_total,
        });

        // Regular files
        for (dest, entry) in &self.manifest.files {
            let (data, mode) = match entry {
                FileEntry::FromFile { source, mode } => {
                    let source_path = self.base_dir.join("files").join(source);
                    let data = std::fs::read(&source_path)
                        .with_context(|| format!("Failed to read file: {}", source_path.display()))?;
                    (data, parse_mode(mode))
                }
                FileEntry::Inline { content, mode } => (content.as_bytes().to_vec(), parse_mode(mode)),
            };
            let size = data.len() as u64;
            archive.add_file(dest, &data, mode);
            emit(BuildEvent::FileAdded {
                path: dest.clone(),
                size,
            });
        }

        // Script files
        for (dest, entry) in &self.manifest.scripts {
            let (data, mode) = match entry {
                FileEntry::FromFile { source, mode } => {
                    let source_path = self.base_dir.join(source);
                    let data = std::fs::read(&source_path)
                        .with_context(|| format!("Failed to read script: {}", source_path.display()))?;
                    (data, parse_mode(mode))
                }
                FileEntry::Inline { content, mode } => (content.as_bytes().to_vec(), parse_mode(mode)),
            };
            let size = data.len() as u64;
            archive.add_file(dest, &data, mode);
            emit(BuildEvent::FileAdded {
                path: dest.clone(),
                size,
            });
        }

        // Add /etc/motd
        let motd = b"Welcome to LevitateOS!\n";
        archive.add_file("etc/motd", motd, 0o644);
        emit(BuildEvent::FileAdded {
            path: "/etc/motd".to_string(),
            size: motd.len() as u64,
        });

        emit(BuildEvent::PhaseComplete {
            name: "Adding files",
        });

        // 5. Create device nodes
        let devices = &self.manifest.devices;
        emit(BuildEvent::PhaseStart {
            name: "Creating devices",
            total: devices.len(),
        });
        for (path, device) in devices {
            let mode = parse_mode(&device.mode);
            match device.dev_type.as_str() {
                "c" => archive.add_char_device(path, mode, device.major, device.minor),
                "b" => archive.add_block_device(path, mode, device.major, device.minor),
                _ => {}
            }
            emit(BuildEvent::DeviceCreated { path: path.clone() });
        }
        emit(BuildEvent::PhaseComplete {
            name: "Creating devices",
        });

        // 6. Write archive
        let output_dir = PathBuf::from("target/initramfs");
        std::fs::create_dir_all(&output_dir)?;
        let output_path = output_dir.join(format!("{}.cpio", self.arch));
        let file = std::fs::File::create(&output_path)?;
        let total_size = archive.write(file)?;

        let duration = start.elapsed();
        emit(BuildEvent::BuildComplete {
            output_path: output_path.clone(),
            total_size,
            duration,
        });

        Ok(output_path)
    }
}
