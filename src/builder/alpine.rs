//! Alpine Linux package extractor
//!
//! Downloads and extracts Alpine packages for use in LevitateOS.
//! Alpine uses musl libc, making packages directly compatible.
//!
//! APK files are just gzipped tarballs with a simple structure.

use anyhow::{bail, Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Alpine Linux mirrors (try in order)
const ALPINE_MIRRORS: &[&str] = &[
    "https://dl-cdn.alpinelinux.org/alpine",
    "https://mirror.leaseweb.com/alpine",
    "https://mirrors.edge.kernel.org/alpine",
];

/// Alpine version to use
/// TEAM_477: Must match distrobox Alpine version (edge) since builds use distrobox libs
const ALPINE_VERSION: &str = "edge";

/// Max retries per mirror
const MAX_RETRIES: u32 = 2;

/// Connection timeout in seconds
const CONNECT_TIMEOUT: u32 = 10;

/// Max download time in seconds
const MAX_TIME: u32 = 60;

/// Get the root directory for extracted Alpine packages
pub fn root_dir(arch: &str) -> PathBuf {
    PathBuf::from(format!("toolchain/alpine-root/{arch}"))
}

/// Get the lib directory for the architecture
pub fn lib_dir(arch: &str) -> PathBuf {
    root_dir(arch).join("lib")
}

/// Get the usr/lib directory for the architecture
pub fn usr_lib_dir(arch: &str) -> PathBuf {
    root_dir(arch).join("usr/lib")
}

/// Convert our arch names to Alpine arch names
fn alpine_arch(arch: &str) -> &str {
    match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => arch,
    }
}

/// Core Wayland packages
pub const WAYLAND_PACKAGES: &[&str] = &[
    "musl",
    "wayland",
    "wayland-libs-client",
    "wayland-libs-server",
    "wayland-libs-cursor",
    "wayland-protocols",
    "libxkbcommon",
];

/// Mesa/Graphics packages for virtio-gpu
pub const MESA_PACKAGES: &[&str] = &[
    "mesa",
    "mesa-gbm",
    "mesa-egl",
    "mesa-gl",
    "mesa-glapi",
    "mesa-gles",
    "mesa-dri-gallium",
    "libdrm",
];

/// Input handling packages
pub const INPUT_PACKAGES: &[&str] = &[
    "libinput",
    "libinput-libs",
    "libevdev",
    "mtdev",
    "eudev-libs",
];

/// Rendering packages
pub const RENDER_PACKAGES: &[&str] = &[
    "pixman",
    "cairo",
    "pango",
    "harfbuzz",
    "freetype",
    "fontconfig",
    "fribidi",
    "glib",
    "pcre2",
];

/// Seat management packages
pub const SEAT_PACKAGES: &[&str] = &["seatd", "libseat"];

/// Font packages for terminal rendering
pub const FONT_PACKAGES: &[&str] = &[
    "font-dejavu",         // Good monospace font
    "fontconfig",
    "libfontenc",
];

/// Additional library dependencies
/// TEAM_477: Full deps for Mesa/GLES2 rendering with v3.21 stable
pub const LIB_PACKAGES: &[&str] = &[
    // Core libraries
    "libffi",
    "libpng",
    "libjpeg-turbo",
    "libwebp",
    "zlib",
    "xz-libs",
    "bzip2",
    "expat",
    "libexpat",          // libexpat.so.1 - XML parser library
    "libxml2",
    // X11/xcb libraries (needed by cairo/pango/mesa)
    "libxcb",
    "xcb-util",
    "xcb-util-wm",
    "xcb-util-renderutil",
    "xcb-util-keysyms",
    "xcb-util-image",
    "xcb-util-cursor",
    "libx11",
    "libxau",
    "libxdmcp",
    "libxext",
    "libxfixes",
    "libxrender",
    "libxcursor",
    "libxi",
    "libxrandr",
    "libxshmfence",
    // JSON for sway
    "json-c",
    // Internationalization
    "libintl",
    // wlroots display-info dependency
    "libdisplay-info",
    // Compression libraries
    "libbz2",
    "brotli-libs",
    "zstd-libs",
    // Font/text rendering
    "graphite2",
    // GCC runtime libraries
    "libgcc",
    "libstdc++",
    // Seat management deps
    "libcap",              // Linux capabilities for elogind
    "libelogind",
    // BSD compatibility
    "libbsd",
    "libmd",               // message digest for libbsd
    // PCI access for DRM
    "libpciaccess",
    // LLVM for Mesa gallium (edge has llvm21)
    "llvm21-libs",
    // SPIRV tools for Mesa shader compilation
    "spirv-tools",
    // ELF handling
    "elfutils",
    // libblkid/libmount for gio
    "libblkid",
    "libmount",
    // libeconf for util-linux
    "libeconf",
];

/// Get all packages needed for Wayland support
pub fn all_wayland_packages() -> Vec<&'static str> {
    let mut packages = Vec::new();
    packages.extend_from_slice(WAYLAND_PACKAGES);
    packages.extend_from_slice(MESA_PACKAGES);
    packages.extend_from_slice(INPUT_PACKAGES);
    packages.extend_from_slice(RENDER_PACKAGES);
    packages.extend_from_slice(SEAT_PACKAGES);
    packages.extend_from_slice(FONT_PACKAGES);
    packages.extend_from_slice(LIB_PACKAGES);
    packages
}

/// Download a file with retry logic across multiple mirrors
fn download_with_retry(url_suffix: &str, dest: &Path, description: &str) -> Result<()> {
    for mirror in ALPINE_MIRRORS {
        let url = format!("{mirror}/{url_suffix}");

        for attempt in 1..=MAX_RETRIES {
            let status = Command::new("curl")
                .args([
                    "-L", "-f", "-s", "-S",
                    "--connect-timeout", &CONNECT_TIMEOUT.to_string(),
                    "--max-time", &MAX_TIME.to_string(),
                    "-o", dest.to_str().unwrap(),
                    &url,
                ])
                .status();

            match status {
                Ok(s) if s.success() => return Ok(()),
                Ok(_) | Err(_) => {
                    if attempt < MAX_RETRIES {
                        eprintln!("    Retry {}/{} for {description}...", attempt + 1, MAX_RETRIES);
                    }
                }
            }
        }
        // Try next mirror
        eprintln!("    Mirror {mirror} failed, trying next...");
    }

    bail!("Failed to download {description} from all mirrors");
}

/// Download APKINDEX for a repository
fn download_apkindex(repo: &str, arch: &str) -> Result<PathBuf> {
    let alpine_arch = alpine_arch(arch);
    let url_suffix = format!("{ALPINE_VERSION}/{repo}/{alpine_arch}/APKINDEX.tar.gz");
    let cache_dir = PathBuf::from(format!("toolchain/alpine-cache/{arch}"));
    std::fs::create_dir_all(&cache_dir)?;

    let index_path = cache_dir.join(format!("APKINDEX-{repo}.tar.gz"));

    // Download if not cached (or stale)
    if !index_path.exists() {
        println!("  Downloading APKINDEX for {repo}...");
        download_with_retry(&url_suffix, &index_path, &format!("APKINDEX-{repo}"))?;
    }

    Ok(index_path)
}

/// Parse APKINDEX to find package filename and dependencies
fn parse_apkindex(index_path: &Path, package: &str) -> Result<Option<(String, Vec<String>)>> {
    // Extract APKINDEX
    let output = Command::new("tar")
        .args(["-xzf", index_path.to_str().unwrap(), "-O", "APKINDEX"])
        .output()
        .context("Failed to extract APKINDEX")?;

    if !output.status.success() {
        bail!("Failed to extract APKINDEX");
    }

    let content = String::from_utf8_lossy(&output.stdout);

    // Parse the index - entries are separated by blank lines
    let mut current_name = String::new();
    let mut current_version = String::new();
    let mut current_deps: Vec<String> = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if line.is_empty() {
            // End of entry - check if this was our package
            if current_name == package {
                found = true;
                break;
            }
            current_name.clear();
            current_version.clear();
            current_deps.clear();
            continue;
        }

        if let Some(name) = line.strip_prefix("P:") {
            current_name = name.to_string();
        } else if let Some(version) = line.strip_prefix("V:") {
            current_version = version.to_string();
        } else if let Some(deps) = line.strip_prefix("D:") {
            // Dependencies are space-separated, may have version constraints
            for dep in deps.split_whitespace() {
                // Strip version constraints like ">=1.0" or "~1.0"
                let dep_name = dep
                    .split(|c| c == '>' || c == '<' || c == '=' || c == '~')
                    .next()
                    .unwrap_or(dep);
                if !dep_name.is_empty() && !dep_name.starts_with("so:") && !dep_name.starts_with("cmd:") {
                    current_deps.push(dep_name.to_string());
                }
            }
        }
    }

    if found && !current_version.is_empty() {
        let filename = format!("{package}-{current_version}.apk");
        Ok(Some((filename, current_deps)))
    } else {
        Ok(None)
    }
}

/// Download a single package
fn download_package(repo: &str, arch: &str, filename: &str) -> Result<PathBuf> {
    let alpine_arch = alpine_arch(arch);
    let url_suffix = format!("{ALPINE_VERSION}/{repo}/{alpine_arch}/{filename}");
    let cache_dir = PathBuf::from(format!("toolchain/alpine-cache/{arch}/packages"));
    std::fs::create_dir_all(&cache_dir)?;

    let pkg_path = cache_dir.join(filename);

    if !pkg_path.exists() {
        println!("  Downloading {filename}...");
        download_with_retry(&url_suffix, &pkg_path, filename)?;
    }

    Ok(pkg_path)
}

/// Extract a package to the root directory
fn extract_package(pkg_path: &Path, root: &Path) -> Result<()> {
    std::fs::create_dir_all(root)?;

    // APK files are gzipped tarballs
    // They contain: .PKGINFO, .SIGN.*, and the actual files
    let status = Command::new("tar")
        .args([
            "-xzf",
            pkg_path.to_str().unwrap(),
            "-C",
            root.to_str().unwrap(),
            "--exclude=.PKGINFO",
            "--exclude=.SIGN.*",
            "--exclude=.pre-install",
            "--exclude=.post-install",
            "--exclude=.pre-deinstall",
            "--exclude=.post-deinstall",
            "--exclude=.trigger",
        ])
        .status()
        .context("Failed to extract package")?;

    if !status.success() {
        bail!("Failed to extract {}", pkg_path.display());
    }

    Ok(())
}

/// Resolve dependencies for a list of packages
fn resolve_dependencies(packages: &[&str], arch: &str) -> Result<Vec<(String, String, String)>> {
    // Download indexes for main and community repos
    let main_index = download_apkindex("main", arch)?;
    let community_index = download_apkindex("community", arch)?;

    let mut resolved: Vec<(String, String, String)> = Vec::new(); // (name, filename, repo)
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = packages.iter().map(|s| s.to_string()).collect();

    while let Some(pkg) = queue.pop() {
        if seen.contains(&pkg) {
            continue;
        }
        seen.insert(pkg.clone());

        // Try main repo first, then community
        let mut found = false;
        for (index, repo) in [(&main_index, "main"), (&community_index, "community")] {
            if let Some((filename, deps)) = parse_apkindex(index, &pkg)? {
                resolved.push((pkg.clone(), filename, repo.to_string()));
                // Add dependencies to queue
                for dep in deps {
                    if !seen.contains(&dep) {
                        queue.push(dep);
                    }
                }
                found = true;
                break;
            }
        }

        if !found {
            // Some packages may be virtual or already satisfied
            // Just warn, don't fail
            eprintln!("  Warning: Package '{}' not found in APKINDEX", pkg);
        }
    }

    Ok(resolved)
}

/// Ensure all Wayland packages are extracted
pub fn ensure_wayland_packages(arch: &str) -> Result<()> {
    let root = root_dir(arch);
    let marker = root.join(".wayland-packages-installed");

    if marker.exists() {
        println!("  Alpine Wayland packages already installed");
        return Ok(());
    }

    println!("  Installing Alpine Wayland packages for {arch}...");

    let packages = all_wayland_packages();
    let resolved = resolve_dependencies(&packages, arch)?;

    println!("  Resolved {} packages (including dependencies)", resolved.len());

    for (name, filename, repo) in &resolved {
        let pkg_path = download_package(repo, arch, filename)?;
        extract_package(&pkg_path, &root)?;
        println!("    Extracted {name}");
    }

    // Create marker file
    std::fs::write(&marker, format!("Installed {} packages\n", resolved.len()))?;

    // Show what we got
    let lib_count = std::fs::read_dir(root.join("lib"))
        .map(|d| d.count())
        .unwrap_or(0);
    let usr_lib_count = std::fs::read_dir(root.join("usr/lib"))
        .map(|d| d.count())
        .unwrap_or(0);

    println!(
        "  Alpine packages installed: {} files in lib/, {} in usr/lib/",
        lib_count, usr_lib_count
    );

    Ok(())
}

/// Get the path to the musl dynamic linker
pub fn dynamic_linker(arch: &str) -> PathBuf {
    let linker_name = match arch {
        "x86_64" => "ld-musl-x86_64.so.1",
        "aarch64" => "ld-musl-aarch64.so.1",
        _ => "ld-musl-x86_64.so.1",
    };
    root_dir(arch).join("lib").join(linker_name)
}

/// Check if Alpine packages are installed
pub fn is_installed(arch: &str) -> bool {
    root_dir(arch).join(".wayland-packages-installed").exists()
}

/// Clean Alpine package cache and extracted files
pub fn clean(arch: &str) -> Result<()> {
    let root = root_dir(arch);
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
        println!("  Removed {}", root.display());
    }

    let cache = PathBuf::from(format!("toolchain/alpine-cache/{arch}"));
    if cache.exists() {
        std::fs::remove_dir_all(&cache)?;
        println!("  Removed {}", cache.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alpine_arch() {
        assert_eq!(alpine_arch("x86_64"), "x86_64");
        assert_eq!(alpine_arch("aarch64"), "aarch64");
    }

    #[test]
    fn test_paths() {
        assert_eq!(
            root_dir("x86_64"),
            PathBuf::from("toolchain/alpine-root/x86_64")
        );
        assert_eq!(
            dynamic_linker("x86_64"),
            PathBuf::from("toolchain/alpine-root/x86_64/lib/ld-musl-x86_64.so.1")
        );
    }
}
