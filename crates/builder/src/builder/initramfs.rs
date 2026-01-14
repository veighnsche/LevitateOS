//! Initramfs CPIO archive builder.

use crate::builder::{brush, glibc, sudo_rs, systemd, util_linux, uutils};
use anyhow::{bail, Context, Result};
use std::path::Path;
use std::process::Command;

pub const ROOT: &str = "build/initramfs";
const CPIO_PATH: &str = "build/initramfs.cpio";

/// Create the initramfs CPIO archive.
pub fn create() -> Result<()> {
    println!("=== Creating initramfs ===");

    // Clean previous build
    if Path::new(ROOT).exists() {
        std::fs::remove_dir_all(ROOT)?;
    }

    create_directories()?;
    glibc::collect()?;
    copy_binaries()?;
    create_init_symlink()?;
    create_coreutils_symlinks()?;
    create_etc_files()?;
    create_systemd_units()?;
    create_cpio()?;

    Ok(())
}

/// Get the CPIO archive path.
pub fn cpio_path() -> &'static str {
    CPIO_PATH
}

fn create_directories() -> Result<()> {
    let dirs = [
        ROOT,
        &format!("{}/bin", ROOT),
        &format!("{}/sbin", ROOT),
        &format!("{}/lib64", ROOT),
        &format!("{}/etc", ROOT),
        &format!("{}/dev", ROOT),
        &format!("{}/proc", ROOT),
        &format!("{}/sys", ROOT),
        &format!("{}/tmp", ROOT),
        &format!("{}/run", ROOT),
        &format!("{}/root", ROOT),
        &format!("{}/home/live", ROOT),
        &format!("{}/var/log", ROOT),
        &format!("{}/usr/lib/systemd/system", ROOT),
        &format!("{}/usr/lib/systemd", ROOT),
        &format!("{}/etc/systemd/system/getty.target.wants", ROOT),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir)?;
    }

    // Compatibility symlinks
    let root = Path::new(ROOT);
    let lib_link = root.join("lib");
    if !lib_link.exists() {
        std::os::unix::fs::symlink("lib64", &lib_link)?;
    }

    // PAM module path: Fedora's libpam looks in /usr/lib64/security/ first
    let usr_lib64 = root.join("usr/lib64");
    std::fs::create_dir_all(&usr_lib64)?;
    let security_link = usr_lib64.join("security");
    if !security_link.exists() {
        std::os::unix::fs::symlink("../../lib64/security", &security_link)?;
    }

    Ok(())
}

fn copy_binaries() -> Result<()> {
    let root = Path::new(ROOT);

    let copies: &[(&str, &str)] = &[
        // Core system
        (uutils::binary_path(), "bin/coreutils"),
        (sudo_rs::sudo_path(), "bin/sudo"),
        (sudo_rs::su_path(), "bin/su"),
        (brush::binary_path(), "bin/brush"),
        (systemd::binary_path(), "sbin/init"),
        (systemd::executor_path(), "usr/lib/systemd/systemd-executor"),
        // Login utilities (util-linux)
        (util_linux::agetty_path(), "sbin/agetty"),
        (util_linux::login_path(), "bin/login"),
        (util_linux::sulogin_path(), "sbin/sulogin"),
        (util_linux::nologin_path(), "sbin/nologin"),
        // Disk utilities (util-linux)
        (util_linux::fdisk_path(), "sbin/fdisk"),
        (util_linux::sfdisk_path(), "sbin/sfdisk"),
        (util_linux::mkfs_path(), "sbin/mkfs"),
        (util_linux::blkid_path(), "sbin/blkid"),
        (util_linux::lsblk_path(), "bin/lsblk"),
        (util_linux::mount_path(), "bin/mount"),
        (util_linux::umount_path(), "bin/umount"),
        (util_linux::losetup_path(), "sbin/losetup"),
    ];

    for (src, dest) in copies {
        let src_path = Path::new(src);
        let dest_path = root.join(dest);

        if src_path.exists() {
            std::fs::copy(src_path, &dest_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&dest_path, std::fs::Permissions::from_mode(0o755))?;
            }
            println!("  Copied: {} -> {}", src, dest);
        } else {
            println!(
                "  Warning: {} not found (run builder <component> first)",
                src
            );
        }
    }

    Ok(())
}

fn create_init_symlink() -> Result<()> {
    let root = Path::new(ROOT);
    let init_link = root.join("init");

    if init_link.exists() {
        std::fs::remove_file(&init_link)?;
    }
    std::os::unix::fs::symlink("sbin/init", &init_link)?;
    println!("  Created /init -> sbin/init symlink");

    Ok(())
}

fn create_coreutils_symlinks() -> Result<()> {
    let root = Path::new(ROOT);

    for cmd in uutils::commands() {
        let link_path = root.join("bin").join(cmd);
        if !link_path.exists() {
            std::os::unix::fs::symlink("coreutils", &link_path)?;
        }
    }
    println!("  Created symlinks for coreutils");

    // Also create /bin/sh -> brush
    let sh_link = root.join("bin/sh");
    if !sh_link.exists() {
        std::os::unix::fs::symlink("brush", &sh_link)?;
    }
    println!("  Created /bin/sh -> brush symlink");

    Ok(())
}

fn create_etc_files() -> Result<()> {
    let root = Path::new(ROOT);

    // passwd: users with x placeholder for password (use shadow)
    std::fs::write(
        root.join("etc/passwd"),
        "root:x:0:0:root:/root:/bin/sh\n\
         live:x:1000:1000:Live User:/home/live:/bin/sh\n\
         nobody:x:65534:65534:Nobody:/:/sbin/nologin\n",
    )?;

    // shadow: password hashes (SHA-512)
    // Default passwords: root="root", live="live"
    // Generated with: openssl passwd -6 -salt saltsalt <password>
    std::fs::write(
        root.join("etc/shadow"),
        "root:$6$saltsalt$bAY90rAsHhyx.bxmKP9FE5UF4jP1iWgjV0ltM6ZJxfYkiIaCExjBZIbfmqmZEWoR65aM.1nFvG7fF3gYOjHpM.:19740:0:99999:7:::\n\
         live:$6$saltsalt$lnz8B.EkP7gx/SsOOLQAcEU/F.7k3CE1I9HTM5hraWcxPafsvSqaJ9s7btu0bk1OOGYbFIG93bLmjZ/qM89J/1:19740:0:99999:7:::\n\
         nobody:!:19740:0:99999:7:::\n",
    )?;

    // Set restrictive permissions on shadow file (required by PAM)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            root.join("etc/shadow"),
            std::fs::Permissions::from_mode(0o600),
        )?;
    }

    // group: user groups
    std::fs::write(
        root.join("etc/group"),
        "root:x:0:\n\
         wheel:x:10:root,live\n\
         live:x:1000:\n\
         nobody:x:65534:\n",
    )?;

    // gshadow: group passwords (empty)
    std::fs::write(
        root.join("etc/gshadow"),
        "root:!::\n\
         wheel:!::root,live\n\
         live:!::\n\
         nobody:!::\n",
    )?;

    std::fs::write(root.join("etc/hostname"), "levitate\n")?;
    std::fs::write(root.join("etc/shells"), "/bin/sh\n/bin/brush\n")?;

    // login.defs: login configuration
    std::fs::write(
        root.join("etc/login.defs"),
        "# Login configuration\n\
         MAIL_DIR /var/mail\n\
         ENV_PATH /usr/local/bin:/usr/bin:/bin\n\
         ENV_SUPATH /usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\n\
         ENCRYPT_METHOD SHA512\n\
         SHA_CRYPT_MIN_ROUNDS 5000\n\
         SHA_CRYPT_MAX_ROUNDS 5000\n",
    )?;

    // securetty: terminals that root can log in from
    std::fs::write(
        root.join("etc/securetty"),
        "console\n\
         tty1\n\
         tty2\n\
         tty3\n\
         tty4\n\
         ttyS0\n",
    )?;

    // nsswitch.conf: tell glibc where to look up users/groups
    std::fs::write(
        root.join("etc/nsswitch.conf"),
        "passwd: files\n\
         group: files\n\
         shadow: files\n",
    )?;

    // PAM configuration
    std::fs::create_dir_all(root.join("etc/pam.d"))?;

    // /etc/pam.d/login - main login PAM config
    std::fs::write(
        root.join("etc/pam.d/login"),
        "auth       required     pam_unix.so shadow nullok\n\
         auth       optional     pam_permit.so\n\
         account    required     pam_unix.so\n\
         account    optional     pam_permit.so\n\
         password   required     pam_unix.so shadow nullok use_authtok\n\
         session    required     pam_unix.so\n",
    )?;

    // /etc/pam.d/other - fallback for services without specific config
    std::fs::write(
        root.join("etc/pam.d/other"),
        "auth       required     pam_unix.so shadow\n\
         auth       optional     pam_permit.so\n\
         account    required     pam_unix.so\n\
         account    optional     pam_permit.so\n\
         password   required     pam_unix.so shadow use_authtok\n\
         session    required     pam_unix.so\n",
    )?;

    // /etc/profile - basic shell environment
    std::fs::write(
        root.join("etc/profile"),
        "export PATH=/usr/local/bin:/usr/bin:/bin:/usr/local/sbin:/usr/sbin:/sbin\n\
         export TERM=${TERM:-vt100}\n\
         export HOME=${HOME:-/root}\n\
         export PS1='\\u@\\h:\\w\\$ '\n",
    )?;

    // Terminfo entries for terminal handling
    let terminfo_dirs = ["usr/share/terminfo/l", "usr/share/terminfo/v"];
    for dir in terminfo_dirs {
        std::fs::create_dir_all(root.join(dir))?;
    }
    // Copy minimal terminfo entries from host
    let terminfo_files = [
        ("/usr/share/terminfo/l/linux", "usr/share/terminfo/l/linux"),
        ("/usr/share/terminfo/v/vt100", "usr/share/terminfo/v/vt100"),
    ];
    for (src, dest) in terminfo_files {
        if Path::new(src).exists() {
            std::fs::copy(src, root.join(dest))?;
        }
    }

    println!("  Created /etc files (with authentication)");
    Ok(())
}

fn create_systemd_units() -> Result<()> {
    let root = Path::new(ROOT);
    let systemd_dir = root.join("usr/lib/systemd/system");
    let getty_wants_dir = root.join("etc/systemd/system/getty.target.wants");

    // sysinit.target - system initialization
    std::fs::write(
        systemd_dir.join("sysinit.target"),
        "[Unit]\n\
         Description=System Initialization\n\
         Documentation=man:systemd.special(7)\n\
         DefaultDependencies=no\n",
    )?;

    // basic.target - basic system ready
    std::fs::write(
        systemd_dir.join("basic.target"),
        "[Unit]\n\
         Description=Basic System\n\
         Documentation=man:systemd.special(7)\n\
         Requires=sysinit.target\n\
         After=sysinit.target\n",
    )?;

    // getty.target - login prompts (after basic, before multi-user)
    std::fs::write(
        systemd_dir.join("getty.target"),
        "[Unit]\n\
         Description=Login Prompts\n\
         Documentation=man:systemd.special(7)\n\
         Requires=basic.target\n\
         After=basic.target\n",
    )?;

    // multi-user.target - full multi-user system
    std::fs::write(
        systemd_dir.join("multi-user.target"),
        "[Unit]\n\
         Description=Multi-User System\n\
         Documentation=man:systemd.special(7)\n\
         Requires=basic.target\n\
         After=basic.target\n\
         Wants=getty.target\n\
         AllowIsolate=yes\n",
    )?;

    // systemd-user-sessions.service - allow user logins
    std::fs::write(
        systemd_dir.join("systemd-user-sessions.service"),
        "[Unit]\n\
         Description=Permit User Sessions\n\
         Documentation=man:systemd-user-sessions.service(8)\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         Type=oneshot\n\
         RemainAfterExit=yes\n\
         ExecStart=/bin/true\n",
    )?;

    // shutdown.target
    std::fs::write(
        systemd_dir.join("shutdown.target"),
        "[Unit]\n\
         Description=System Shutdown\n\
         Documentation=man:systemd.special(7)\n\
         DefaultDependencies=no\n",
    )?;

    // getty@.service (template for virtual terminals)
    std::fs::write(
        systemd_dir.join("getty@.service"),
        "[Unit]\n\
         Description=Getty on %I\n\
         Documentation=man:agetty(8)\n\
         After=systemd-user-sessions.service\n\
         \n\
         [Service]\n\
         ExecStart=-/sbin/agetty -o '-p -- \\\\u' --noclear %I linux\n\
         Type=idle\n\
         Restart=always\n\
         RestartSec=0\n\
         UtmpIdentifier=%I\n\
         TTYPath=/dev/%I\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
         TTYVTDisallocate=yes\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n\
         \n\
         [Install]\n\
         WantedBy=getty.target\n",
    )?;

    // serial-getty@.service (for serial consoles - no udev dependency)
    std::fs::write(
        systemd_dir.join("serial-getty@.service"),
        "[Unit]\n\
         Description=Serial Getty on %I\n\
         Documentation=man:agetty(8)\n\
         After=systemd-user-sessions.service\n\
         \n\
         [Service]\n\
         ExecStart=-/sbin/agetty --autologin root --keep-baud 115200,38400,9600 %I vt100\n\
         Type=idle\n\
         Restart=always\n\
         RestartSec=0\n\
         UtmpIdentifier=%I\n\
         TTYPath=/dev/%I\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n\
         \n\
         [Install]\n\
         WantedBy=getty.target\n",
    )?;

    // Create getty.target.wants symlinks
    let getty_tty1 = getty_wants_dir.join("getty@tty1.service");
    if !getty_tty1.exists() {
        std::os::unix::fs::symlink(
            "/usr/lib/systemd/system/getty@.service",
            &getty_tty1,
        )?;
    }

    let serial_getty = getty_wants_dir.join("serial-getty@ttyS0.service");
    if !serial_getty.exists() {
        std::os::unix::fs::symlink(
            "/usr/lib/systemd/system/serial-getty@.service",
            &serial_getty,
        )?;
    }

    // rescue.target
    std::fs::write(
        systemd_dir.join("rescue.target"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         Documentation=man:systemd.special(7)\n\
         Requires=sysinit.target rescue.service\n\
         After=sysinit.target rescue.service\n\
         AllowIsolate=yes\n",
    )?;

    // rescue.service
    std::fs::write(
        systemd_dir.join("rescue.service"),
        "[Unit]\n\
         Description=Rescue Shell\n\
         DefaultDependencies=no\n\
         After=sysinit.target\n\
         \n\
         [Service]\n\
         Environment=HOME=/root\n\
         Environment=TERM=linux\n\
         WorkingDirectory=/root\n\
         ExecStart=-/bin/sh\n\
         Type=idle\n\
         StandardInput=tty\n\
         StandardOutput=tty\n\
         StandardError=tty\n\
         TTYPath=/dev/console\n\
         TTYReset=yes\n\
         TTYVHangup=yes\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n\
         SendSIGHUP=yes\n",
    )?;

    // emergency.target
    std::fs::write(
        systemd_dir.join("emergency.target"),
        "[Unit]\n\
         Description=Emergency Shell\n\
         Documentation=man:systemd.special(7)\n\
         Requires=emergency.service\n\
         After=emergency.service\n\
         AllowIsolate=yes\n",
    )?;

    // emergency.service
    std::fs::write(
        systemd_dir.join("emergency.service"),
        "[Unit]\n\
         Description=Emergency Shell\n\
         DefaultDependencies=no\n\
         \n\
         [Service]\n\
         Environment=HOME=/root\n\
         WorkingDirectory=/root\n\
         ExecStart=-/bin/sh\n\
         Type=idle\n\
         StandardInput=tty-force\n\
         StandardOutput=inherit\n\
         StandardError=inherit\n\
         KillMode=process\n\
         IgnoreSIGPIPE=no\n",
    )?;

    // default.target symlink -> multi-user.target
    let default_target = systemd_dir.join("default.target");
    if default_target.exists() {
        std::fs::remove_file(&default_target)?;
    }
    std::os::unix::fs::symlink("multi-user.target", &default_target)?;

    println!("  Created systemd unit files (with getty)");
    Ok(())
}

fn create_cpio() -> Result<()> {
    // Use fakeroot to make all files appear owned by root:root
    // This is required for PAM to accept the shadow file
    let output = Command::new("fakeroot")
        .args([
            "sh",
            "-c",
            &format!(
                "cd {} && find . | cpio -o -H newc > ../initramfs.cpio",
                ROOT
            ),
        ])
        .output()
        .context("Failed to create CPIO (is fakeroot installed?)")?;

    if !output.status.success() {
        bail!(
            "CPIO creation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let size = std::fs::metadata(CPIO_PATH)?.len();
    println!(
        "\n  Created: {} ({:.1} MB)",
        CPIO_PATH,
        size as f64 / 1_000_000.0
    );

    Ok(())
}
