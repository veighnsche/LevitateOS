//! Initramfs builder - constructs CPIO from manifest
//!
//! TEAM_474: Event-driven builder for TUI progress reporting.

use super::cpio::CpioArchive;
use super::manifest::{parse_mode, FileEntry, Manifest};
use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// Build events emitted during construction
#[derive(Clone, Debug)]
#[allow(dead_code)] // Fields read by TUI pattern matching
pub enum BuildEvent {
    PhaseStart { name: &'static str, total: usize },
    PhaseComplete { name: &'static str },
    DirectoryCreated { path: String },
    BinaryAdded { path: String, size: u64 },
    SymlinkCreated { link: String, target: String },
    FileAdded { path: String, size: u64 },
    DeviceCreated { path: String },
    TreeCopied { dest: String, files: usize },
    BuildComplete {
        output_path: PathBuf,
        total_size: u64,
        duration: Duration,
    },
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
            let data = std::fs::read(&source_path).with_context(|| {
                format!(
                    "Failed to read binary '{}': {}",
                    name,
                    source_path.display()
                )
            })?;
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
                    let data = std::fs::read(&source_path).with_context(|| {
                        format!("Failed to read file: {}", source_path.display())
                    })?;
                    (data, parse_mode(mode))
                }
                FileEntry::Inline { content, mode } => {
                    (content.as_bytes().to_vec(), parse_mode(mode))
                }
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
                    let data = std::fs::read(&source_path).with_context(|| {
                        format!("Failed to read script: {}", source_path.display())
                    })?;
                    (data, parse_mode(mode))
                }
                FileEntry::Inline { content, mode } => {
                    (content.as_bytes().to_vec(), parse_mode(mode))
                }
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

        // 6. Copy directory trees
        let trees = &self.manifest.trees;
        if !trees.is_empty() {
            emit(BuildEvent::PhaseStart {
                name: "Copying trees",
                total: trees.len(),
            });
            for (dest, source) in trees {
                let source_path = PathBuf::from(source);
                if source_path.exists() {
                    let files = self.copy_tree(&mut archive, &source_path, dest)?;
                    emit(BuildEvent::TreeCopied {
                        dest: dest.clone(),
                        files,
                    });
                }
            }
            emit(BuildEvent::PhaseComplete {
                name: "Copying trees",
            });
        }

        // 7. Write archive
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

    /// Recursively copy a directory tree into the archive
    fn copy_tree(&self, archive: &mut CpioArchive, src: &Path, dest_prefix: &str) -> Result<usize> {
        let mut count = 0;

        for entry in std::fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let name = path.file_name().unwrap().to_string_lossy();
            let dest = format!("{}/{}", dest_prefix, name);

            if path.is_symlink() {
                let target = std::fs::read_link(&path)?;
                archive.add_symlink(&dest, &target.to_string_lossy());
                count += 1;
            } else if path.is_dir() {
                archive.add_directory(&dest, 0o755);
                count += self.copy_tree(archive, &path, &dest)?;
            } else if path.is_file() {
                let data = std::fs::read(&path)?;
                let mode = std::fs::metadata(&path)?.permissions().mode() & 0o7777;
                archive.add_file(&dest, &data, mode);
                count += 1;
            }
        }

        Ok(count)
    }
}
