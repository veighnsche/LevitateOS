use anyhow::{bail, Context, Result};
use clap::Subcommand;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::disk;

// TEAM_459: Simplified - BusyBox is the only external app now
#[derive(Subcommand)]
pub enum BuildCommands {
    /// Build everything (Kernel + Userspace + Disk + BusyBox)
    All,
    /// Build kernel only
    Kernel,
    /// Build userspace only
    Userspace,
    /// Build initramfs only
    Initramfs,
    /// Build bootable Limine ISO
    Iso,
    /// Build BusyBox - provides init, shell, and 300+ utilities
    Busybox,
}

// TEAM_435: Replaced Eyra with c-gull sysroot approach
// TEAM_438: Uses apps registry for external app builds
// TEAM_444: Migrated to musl - much simpler now!
pub fn build_all(arch: &str) -> Result<()> {
    // Ensure musl target is installed (replaces sysroot build)
    super::sysroot::ensure_rust_musl_target(arch)?;

    // Build all external Rust apps (coreutils, brush, etc.) if not present
    super::apps::ensure_all_built(arch)?;

    // Build C apps if musl-gcc is available (optional)
    if super::c_apps::musl_gcc_available() {
        for app in super::c_apps::C_APPS {
            if !app.exists(arch) {
                // Don't fail build_all if C app build fails - it's optional
                if let Err(e) = app.build(arch) {
                    println!("⚠️  Optional C app {} failed to build: {}", app.name, e);
                }
            }
        }
    } else {
        println!("ℹ️  musl-gcc not found, skipping C apps (dash). Install musl-tools to enable.");
    }

    // TEAM_073: Build userspace first
    build_userspace(arch)?;
    // TEAM_451: Use BusyBox initramfs (replaces old init + dash + coreutils)
    create_busybox_initramfs(arch)?;
    // TEAM_121: Ensure disk image is populated
    disk::install_userspace_to_disk(arch)?;

    build_kernel_with_features(&[], arch)
}

pub fn build_kernel_only(arch: &str) -> Result<()> {
    build_kernel_with_features(&[], arch)
}

/// Build kernel with verbose feature for behavior testing (Rule 4: Silence is Golden)
pub fn build_kernel_verbose(arch: &str) -> Result<()> {
    build_kernel_with_features(&["verbose"], arch)
}

pub fn build_userspace(arch: &str) -> Result<()> {
    println!("Building userspace workspace for {}...", arch);
    
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // TEAM_120: Build the entire userspace workspace
    // We build in-place now as the workspace isolation issues should be resolved
    // by individual build.rs scripts and correct linker arguments.
    let status = Command::new("cargo")
        .current_dir("crates/userspace")
        .args([
            "build",
            "--release",
            "--workspace",
            "--target", target,
        ])
        .status()
        .context("Failed to build userspace workspace")?;

    if !status.success() {
        bail!("Userspace workspace build failed");
    }

    Ok(())
}

// TEAM_435: Uses c-gull sysroot binaries instead of Eyra
// TEAM_444: Migrated to musl - Rust apps use musl target, C apps use musl-gcc
pub fn create_initramfs(arch: &str) -> Result<()> {
    println!("Creating initramfs for {}...", arch);
    let root = PathBuf::from("initrd_root");

    // TEAM_292: Always clean initrd_root to ensure correct arch binaries
    // Without this, stale binaries from other architectures persist
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    // 1. Create content
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;

    // 2. Copy userspace binaries (init, shell - bare-metal)
    let binaries = crate::get_binaries(arch)?;
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };
    print!("📦 Creating initramfs ({} binaries)... ", binaries.len());
    let mut count = 0;
    for bin in &binaries {
        let src = PathBuf::from(format!("crates/userspace/target/{}/release/{}", target, bin));
        if src.exists() {
            std::fs::copy(&src, root.join(bin))?;
            count += 1;
        }
    }

    // TEAM_438: Use apps registry for external apps - fail fast on required, skip optional
    for app in super::apps::APPS {
        if app.required {
            // Required apps must exist - fail fast with helpful message
            let src = app.require(arch)?;
            std::fs::copy(&src, root.join(app.binary))?;
            count += 1;

            // Create symlinks for multi-call binaries
            for symlink_name in app.symlinks {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    let link_path = root.join(symlink_name);
                    let _ = std::fs::remove_file(&link_path);
                    symlink(app.binary, &link_path)?;
                }
                #[cfg(not(unix))]
                {
                    std::fs::copy(&src, root.join(symlink_name))?;
                }
            }

            if app.symlinks.is_empty() {
                println!("  📦 Added {}", app.name);
            } else {
                println!("  📦 Added {} + {} symlinks", app.name, app.symlinks.len());
            }
        } else {
            // Optional apps - include if built, otherwise inform user
            if app.exists(arch) {
                let src = app.output_path(arch);
                std::fs::copy(&src, root.join(app.binary))?;
                count += 1;
                println!("  📦 Added {} (optional)", app.name);
            } else {
                println!("  ℹ️  {} not found (optional). Run 'cargo xtask build {}' to include it.", app.name, app.name);
            }
        }
    }

    // TEAM_444: Include C apps (dash, etc.) if built
    for app in super::c_apps::C_APPS {
        if app.exists(arch) {
            let src = app.output_path(arch);
            let binary_name = std::path::Path::new(app.binary)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| app.name.to_string());
            std::fs::copy(&src, root.join(&binary_name))?;
            count += 1;
            println!("  📦 Added {} (C)", app.name);
        }
    }

    println!("[DONE] ({} added)", count);

    // 3. Create CPIO archive
    // TEAM_327: Use arch-specific filename to prevent cross-arch contamination
    // usage: find . | cpio -o -H newc > ../initramfs_{arch}.cpio
    let cpio_filename = format!("initramfs_{}.cpio", arch);
    let cpio_file = std::fs::File::create(&cpio_filename)?;
    
    let find = Command::new("find")
        .current_dir(&root)
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run find")?;

    let mut cpio = Command::new("cpio")
        .current_dir(&root)
        .args(["-o", "-H", "newc"])
        .stdin(find.stdout.unwrap())
        .stdout(cpio_file)
        .spawn()
        .context("Failed to run cpio")?;

    let status = cpio.wait()?;
    if !status.success() {
        bail!("cpio failed");
    }

    Ok(())
}

/// TEAM_451: Create BusyBox-based initramfs
/// Single binary provides init, shell, and 300+ utilities
pub fn create_busybox_initramfs(arch: &str) -> Result<()> {
    println!("📦 Creating BusyBox initramfs for {}...", arch);
    
    // Require BusyBox to be built
    let busybox_path = super::busybox::require(arch)?;
    
    let root = PathBuf::from("initrd_root");

    // Clean and create directory structure
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir_all(&root)?;
    std::fs::create_dir_all(root.join("bin"))?;
    std::fs::create_dir_all(root.join("sbin"))?;
    std::fs::create_dir_all(root.join("etc"))?;
    std::fs::create_dir_all(root.join("proc"))?;
    std::fs::create_dir_all(root.join("sys"))?;
    std::fs::create_dir_all(root.join("tmp"))?;
    std::fs::create_dir_all(root.join("dev"))?;
    std::fs::create_dir_all(root.join("root"))?;

    // Copy BusyBox binary
    std::fs::copy(&busybox_path, root.join("bin/busybox"))?;
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(root.join("bin/busybox"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(root.join("bin/busybox"), perms)?;
    }

    // Create symlinks for all applets
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        
        for (applet, dir) in super::busybox::applets() {
            let link_path = root.join(dir).join(applet);
            let target = if *dir == "sbin" {
                "../bin/busybox"
            } else {
                "busybox"
            };
            let _ = std::fs::remove_file(&link_path);
            symlink(target, &link_path)?;
        }
    }
    
    // Create /init as a copy of busybox (kernel entry point)
    // TEAM_451: Can't use symlink - kernel ELF loader doesn't follow symlinks
    let _ = std::fs::remove_file(root.join("init"));
    std::fs::copy(&busybox_path, root.join("init"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(root.join("init"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(root.join("init"), perms)?;
    }

    // Create /etc/inittab (TEAM_460: changed to wait so exit works)
    let inittab = r#"# LevitateOS BusyBox init configuration
# TEAM_451: Generated by xtask
# TEAM_460: Changed respawn to wait so 'exit' terminates shell

# System initialization
::sysinit:/bin/echo "LevitateOS (BusyBox) starting..."
::sysinit:/bin/mount -t proc proc /proc
::sysinit:/bin/mount -t sysfs sysfs /sys

# Note: There's a kernel bug where initramfs subdirectory contents don't show in readdir
# Files CAN be read by exact path, but ls won't list them
# Run: sh /root/test-core.sh to test coreutils (has some known failures due to this)

# Start interactive shell
::wait:-/bin/ash

# Handle Ctrl+Alt+Del
::ctrlaltdel:/sbin/reboot

# Shutdown hooks
::shutdown:/bin/echo "System shutting down..."
"#;
    std::fs::write(root.join("etc/inittab"), inittab)?;

    // Create /etc/passwd
    let passwd = "root:x:0:0:root:/root:/bin/ash\n";
    std::fs::write(root.join("etc/passwd"), passwd)?;

    // Create /etc/group
    let group = "root:x:0:\n";
    std::fs::write(root.join("etc/group"), group)?;

    // Create /etc/profile
    let profile = r#"export PATH=/bin:/sbin
export HOME=/root
export PS1='LevitateOS# '
alias ll='ls -la'
"#;
    std::fs::write(root.join("etc/profile"), profile)?;

    // Create sample files
    std::fs::write(root.join("etc/motd"), "Welcome to LevitateOS!\n")?;
    std::fs::write(root.join("root/hello.txt"), "Hello from BusyBox initramfs!\n")?;

    // TEAM_459: Test script to verify ash shell works
    let test_script = r#"#!/bin/ash
# Test script to verify ash shell functionality

echo "=== ASH SHELL TEST ==="
echo ""

# Test 1: Echo
echo "[TEST 1] echo: PASS"

# Test 2: Variables
VAR="hello"
if [ "$VAR" = "hello" ]; then
    echo "[TEST 2] variables: PASS"
else
    echo "[TEST 2] variables: FAIL"
fi

# Test 3: Command substitution
RESULT=$(echo "test")
if [ "$RESULT" = "test" ]; then
    echo "[TEST 3] command substitution: PASS"
else
    echo "[TEST 3] command substitution: FAIL"
fi

# Test 4: Conditionals
if true; then
    echo "[TEST 4] conditionals: PASS"
else
    echo "[TEST 4] conditionals: FAIL"
fi

# Test 5: For loop
COUNT=0
for i in 1 2 3; do
    COUNT=$((COUNT + 1))
done
if [ "$COUNT" = "3" ]; then
    echo "[TEST 5] for loop: PASS"
else
    echo "[TEST 5] for loop: FAIL"
fi

# Test 6: Arithmetic
X=$((2 + 3))
if [ "$X" = "5" ]; then
    echo "[TEST 6] arithmetic: PASS"
else
    echo "[TEST 6] arithmetic: FAIL"
fi

# Test 7: Cat file
if cat /root/hello.txt > /dev/null 2>&1; then
    echo "[TEST 7] cat file: PASS"
else
    echo "[TEST 7] cat file: FAIL"
fi

# Test 8: Ls directory
if ls / > /dev/null 2>&1; then
    echo "[TEST 8] ls directory: PASS"
else
    echo "[TEST 8] ls directory: FAIL"
fi

echo ""
echo "=== ALL TESTS COMPLETE ==="
"#;
    std::fs::write(root.join("root/test.sh"), test_script)?;
    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(root.join("root/test.sh"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(root.join("root/test.sh"), perms)?;
    }

    // TEAM_460: Comprehensive coreutils test suite - deliberate dependency order
    // TEAM_465: Added phase selection support
    // Run with: sh /root/test-core.sh [PHASE]
    let test_core = r#"#!/bin/ash
#=============================================================================
# LevitateOS BusyBox Coreutils Test Suite
#
# Tests most-used coreutils in DEPENDENCY ORDER:
#   Phase 1: Output (echo, printf) - no deps
#   Phase 2: Directory creation (mkdir) - needed for test folder
#   Phase 3: File creation (touch, echo >) - need dirs first
#   Phase 4: File reading (cat, head, tail, wc) - need files first
#   Phase 5: File manipulation (cp, mv, rm, ln) - need files first
#   Phase 6: Directory listing (ls, pwd, basename, dirname)
#   Phase 7: Text processing (grep, sed, tr, cut, sort, uniq)
#   Phase 8: Conditionals (test, true, false, expr)
#   Phase 9: Iteration (seq, xargs)
#   Phase 10: System info (uname, id, hostname)
#   Phase 11: Pipes and redirection
#   Phase 12: Command substitution
#   Phase 13: Find (needs directory tree)
#
# Test folder: /root/coretest (created fresh, cleaned up after)
#
# Usage: test-core.sh [PHASE]
#   PHASE: 1-13, "all" (default), or range like "2-5"
#   Examples:
#     test-core.sh         # Run all phases
#     test-core.sh 2       # Run only phase 2
#     test-core.sh 2-5     # Run phases 2 through 5
#=============================================================================

# Parse phase argument
PHASE_ARG="${1:-all}"

# Determine which phases to run
run_phase() {
    phase=$1
    if [ "$PHASE_ARG" = "all" ]; then
        return 0  # run all
    elif echo "$PHASE_ARG" | grep -q "-"; then
        # Range like "2-5"
        start=$(echo "$PHASE_ARG" | cut -d- -f1)
        end=$(echo "$PHASE_ARG" | cut -d- -f2)
        [ "$phase" -ge "$start" ] && [ "$phase" -le "$end" ]
        return $?
    else
        # Single phase
        [ "$phase" -eq "$PHASE_ARG" ]
        return $?
    fi
}

PASS=0
FAIL=0

# Test result functions
pass() {
    PASS=$((PASS + 1))
    echo "  [PASS] $1"
}

fail() {
    FAIL=$((FAIL + 1))
    echo "  [FAIL] $1 - $2"
}

# Assertion helpers
check_eq() {
    # $1=name $2=expected $3=actual
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected '$2', got '$3'"
    fi
}

check_exit() {
    # $1=name $2=expected_exit $3=actual_exit
    if [ "$2" = "$3" ]; then
        pass "$1"
    else
        fail "$1" "expected exit $2, got $3"
    fi
}

check_file_exists() {
    # $1=name $2=path
    if [ -e "$2" ]; then
        pass "$1"
    else
        fail "$1" "file '$2' does not exist"
    fi
}

check_file_gone() {
    # $1=name $2=path
    if [ ! -e "$2" ]; then
        pass "$1"
    else
        fail "$1" "file '$2' still exists"
    fi
}

check_contains() {
    # $1=name $2=needle $3=haystack
    case "$3" in
        *"$2"*) pass "$1" ;;
        *) fail "$1" "does not contain '$2'" ;;
    esac
}

#=============================================================================
echo "========================================"
echo " LevitateOS Coreutils Test Suite"
echo "========================================"
[ "$PHASE_ARG" != "all" ] && echo " Running phase(s): $PHASE_ARG"
echo ""

# Setup test directory variable (used by phases 2+)
TEST_DIR="/root/coretest"

#-----------------------------------------------------------------------------
if run_phase 1; then
echo "[Phase 1] Basic Output (echo, printf)"
echo "----------------------------------------"

OUT=$(echo "hello")
check_eq "echo basic" "hello" "$OUT"

OUT=$(echo -n "no-newline")
check_eq "echo -n" "no-newline" "$OUT"

OUT=$(echo "a b c")
check_eq "echo with spaces" "a b c" "$OUT"

OUT=$(printf "%s" "test")
check_eq "printf %s" "test" "$OUT"

OUT=$(printf "%d" 42)
check_eq "printf %d" "42" "$OUT"

OUT=$(printf "x=%d y=%s" 10 "hi")
check_eq "printf multi" "x=10 y=hi" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 2; then
echo "[Phase 2] Directory Creation (mkdir)"
echo "----------------------------------------"

# Clean any previous run
rm -rf "$TEST_DIR" 2>/dev/null

mkdir "$TEST_DIR"
check_file_exists "mkdir basic" "$TEST_DIR"

mkdir "$TEST_DIR/sub1"
check_file_exists "mkdir nested" "$TEST_DIR/sub1"

mkdir -p "$TEST_DIR/deep/nested/path"
check_file_exists "mkdir -p deep" "$TEST_DIR/deep/nested/path"

# Move into test directory
cd "$TEST_DIR"

echo ""
fi

# Ensure test dir exists for phases 3+ (even if phase 2 was skipped)
if [ -d "$TEST_DIR" ]; then
    cd "$TEST_DIR"
fi

#-----------------------------------------------------------------------------
if run_phase 3; then
echo "[Phase 3] File Creation (touch, echo >)"
echo "----------------------------------------"

# Create test dir if phase 2 was skipped
if [ ! -d "$TEST_DIR" ]; then
    mkdir -p "$TEST_DIR"
    cd "$TEST_DIR"
fi

touch file1.txt
check_file_exists "touch create" "file1.txt"

echo "content" > file2.txt
check_file_exists "echo > create" "file2.txt"

echo "line1" > multi.txt
echo "line2" >> multi.txt
echo "line3" >> multi.txt
check_file_exists "echo >> append" "multi.txt"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 4; then
echo "[Phase 4] File Reading (cat, head, tail, wc)"
echo "----------------------------------------"

OUT=$(cat file2.txt)
check_eq "cat single line" "content" "$OUT"

OUT=$(cat multi.txt)
EXPECTED="line1
line2
line3"
check_eq "cat multi-line" "$EXPECTED" "$OUT"

OUT=$(head -n 1 multi.txt)
check_eq "head -n 1" "line1" "$OUT"

OUT=$(head -n 2 multi.txt)
EXPECTED="line1
line2"
check_eq "head -n 2" "$EXPECTED" "$OUT"

OUT=$(tail -n 1 multi.txt)
check_eq "tail -n 1" "line3" "$OUT"

OUT=$(tail -n 2 multi.txt)
EXPECTED="line2
line3"
check_eq "tail -n 2" "$EXPECTED" "$OUT"

OUT=$(wc -l < multi.txt)
OUT=$(echo $OUT)  # trim whitespace
check_eq "wc -l" "3" "$OUT"

OUT=$(echo -n "hello" | wc -c)
OUT=$(echo $OUT)
check_eq "wc -c" "5" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 5; then
echo "[Phase 5] File Manipulation (cp, mv, rm, ln)"
echo "----------------------------------------"

# cp
echo "original" > orig.txt
cp orig.txt copy.txt
check_file_exists "cp creates dest" "copy.txt"
OUT=$(cat copy.txt)
check_eq "cp preserves content" "original" "$OUT"

# mv
echo "moveme" > src.txt
mv src.txt dst.txt
check_file_gone "mv removes source" "src.txt"
check_file_exists "mv creates dest" "dst.txt"
OUT=$(cat dst.txt)
check_eq "mv preserves content" "moveme" "$OUT"

# rm
echo "deleteme" > del.txt
rm del.txt
check_file_gone "rm deletes file" "del.txt"

# rm -r
mkdir -p rmdir/sub
touch rmdir/sub/file.txt
rm -r rmdir
check_file_gone "rm -r deletes tree" "rmdir"

# ln -s (symlink)
echo "target" > target.txt
ln -s target.txt link.txt
check_file_exists "ln -s creates link" "link.txt"
OUT=$(cat link.txt)
check_eq "ln -s readable" "target" "$OUT"

# rmdir
mkdir emptydir
rmdir emptydir
check_file_gone "rmdir removes empty" "emptydir"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 6; then
echo "[Phase 6] Directory Listing (ls, pwd, basename, dirname)"
echo "----------------------------------------"

OUT=$(pwd)
check_contains "pwd contains test" "coretest" "$OUT"

mkdir lsdir
touch lsdir/aaa.txt lsdir/bbb.txt lsdir/ccc.txt

OUT=$(ls lsdir)
check_contains "ls shows aaa.txt" "aaa.txt" "$OUT"
check_contains "ls shows bbb.txt" "bbb.txt" "$OUT"

touch lsdir/.hidden
OUT=$(ls -a lsdir)
check_contains "ls -a shows hidden" ".hidden" "$OUT"

OUT=$(ls -l lsdir/aaa.txt)
check_contains "ls -l shows perms" "rw" "$OUT"

OUT=$(basename /path/to/file.txt)
check_eq "basename" "file.txt" "$OUT"

OUT=$(dirname /path/to/file.txt)
check_eq "dirname" "/path/to" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 7; then
echo "[Phase 7] Text Processing (grep, sed, tr, cut, sort, uniq)"
echo "----------------------------------------"

# Setup test file
echo "apple" > fruits.txt
echo "banana" >> fruits.txt
echo "cherry" >> fruits.txt
echo "apricot" >> fruits.txt

# grep
OUT=$(grep "banana" fruits.txt)
check_eq "grep exact" "banana" "$OUT"

OUT=$(grep "^a" fruits.txt)
EXPECTED="apple
apricot"
check_eq "grep pattern" "$EXPECTED" "$OUT"

OUT=$(grep -c "a" fruits.txt)
check_eq "grep -c count" "4" "$OUT"

OUT=$(grep -v "a" fruits.txt)
check_eq "grep -v invert" "cherry" "$OUT"

# sed
OUT=$(echo "hello world" | sed 's/world/earth/')
check_eq "sed substitute" "hello earth" "$OUT"

OUT=$(echo "aaa" | sed 's/a/b/g')
check_eq "sed global" "bbb" "$OUT"

# tr
OUT=$(echo "hello" | tr 'a-z' 'A-Z')
check_eq "tr uppercase" "HELLO" "$OUT"

OUT=$(echo "hello" | tr -d 'l')
check_eq "tr delete" "heo" "$OUT"

OUT=$(echo "a  b   c" | tr -s ' ')
check_eq "tr squeeze" "a b c" "$OUT"

# cut
echo "a:b:c:d" > cut.txt
OUT=$(cut -d: -f2 cut.txt)
check_eq "cut field 2" "b" "$OUT"

OUT=$(cut -d: -f2,4 cut.txt)
check_eq "cut fields 2,4" "b:d" "$OUT"

# sort
echo "cherry" > sort.txt
echo "apple" >> sort.txt
echo "banana" >> sort.txt
OUT=$(sort sort.txt)
EXPECTED="apple
banana
cherry"
check_eq "sort alpha" "$EXPECTED" "$OUT"

# uniq
echo "a" > uniq.txt
echo "a" >> uniq.txt
echo "b" >> uniq.txt
echo "b" >> uniq.txt
echo "a" >> uniq.txt
OUT=$(uniq uniq.txt)
EXPECTED="a
b
a"
check_eq "uniq adjacent" "$EXPECTED" "$OUT"

OUT=$(sort uniq.txt | uniq)
EXPECTED="a
b"
check_eq "sort | uniq" "$EXPECTED" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 8; then
echo "[Phase 8] Conditionals (test, true, false, expr)"
echo "----------------------------------------"

# test numeric
[ 5 -eq 5 ]
check_exit "test -eq true" "0" "$?"

[ 5 -eq 3 ]
check_exit "test -eq false" "1" "$?"

[ 5 -gt 3 ]
check_exit "test -gt" "0" "$?"

[ 3 -lt 5 ]
check_exit "test -lt" "0" "$?"

# test string
[ "abc" = "abc" ]
check_exit "test string =" "0" "$?"

[ "abc" != "xyz" ]
check_exit "test string !=" "0" "$?"

[ -n "nonempty" ]
check_exit "test -n" "0" "$?"

[ -z "" ]
check_exit "test -z" "0" "$?"

# test file
[ -f file1.txt ]
check_exit "test -f file" "0" "$?"

[ -d lsdir ]
check_exit "test -d dir" "0" "$?"

[ -e link.txt ]
check_exit "test -e exists" "0" "$?"

# true/false
true
check_exit "true" "0" "$?"

false
check_exit "false" "1" "$?"

# expr
OUT=$(expr 2 + 3)
check_eq "expr add" "5" "$OUT"

OUT=$(expr 10 - 4)
check_eq "expr sub" "6" "$OUT"

OUT=$(expr 3 \* 4)
check_eq "expr mul" "12" "$OUT"

OUT=$(expr 10 / 3)
check_eq "expr div" "3" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 9; then
echo "[Phase 9] Iteration (seq, xargs)"
echo "----------------------------------------"

OUT=$(seq 3)
EXPECTED="1
2
3"
check_eq "seq 3" "$EXPECTED" "$OUT"

OUT=$(seq 2 5)
EXPECTED="2
3
4
5"
check_eq "seq 2 5" "$EXPECTED" "$OUT"

# xargs
echo "a b c" | xargs -n1 echo > xargs.txt
OUT=$(cat xargs.txt)
EXPECTED="a
b
c"
check_eq "xargs -n1" "$EXPECTED" "$OUT"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 10; then
echo "[Phase 10] System Info (uname, id, hostname)"
echo "----------------------------------------"

OUT=$(uname -s)
[ -n "$OUT" ]
check_exit "uname -s" "0" "$?"

OUT=$(uname -m)
[ -n "$OUT" ]
check_exit "uname -m" "0" "$?"

OUT=$(id -u)
check_eq "id -u root" "0" "$OUT"

OUT=$(id -g)
check_eq "id -g root" "0" "$OUT"

# hostname might not be set
hostname >/dev/null 2>&1
check_exit "hostname runs" "0" "$?"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 11; then
echo "[Phase 11] Pipes and Redirection"
echo "----------------------------------------"

# Basic pipe
OUT=$(echo "hello" | cat)
check_eq "pipe basic" "hello" "$OUT"

# Multi-stage pipe
OUT=$(echo "HELLO" | tr 'A-Z' 'a-z' | cat)
check_eq "pipe multi" "hello" "$OUT"

# Pipe with grep
OUT=$(echo -e "a\nb\nc" | grep "b")
check_eq "pipe grep" "b" "$OUT"

# Redirect output
echo "redir-test" > redir.txt
OUT=$(cat redir.txt)
check_eq "redirect >" "redir-test" "$OUT"

# Append
echo "line1" > append.txt
echo "line2" >> append.txt
OUT=$(cat append.txt)
EXPECTED="line1
line2"
check_eq "redirect >>" "$EXPECTED" "$OUT"

# tee
echo "tee-test" | tee tee.txt > /dev/null
OUT=$(cat tee.txt)
check_eq "tee" "tee-test" "$OUT"

# /dev/null
echo "gone" > /dev/null
check_exit "/dev/null write" "0" "$?"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 12; then
echo "[Phase 12] Command Substitution"
echo "----------------------------------------"

# Basic
VAR=$(echo "value")
check_eq "cmd subst basic" "value" "$VAR"

# Nested
INNER=$(echo "inner")
OUTER=$(echo "got $INNER")
check_eq "cmd subst nested" "got inner" "$OUTER"

# In arithmetic
X=$((2 + 3))
check_eq "arith basic" "5" "$X"

X=$((2 + 3 * 4))
check_eq "arith precedence" "14" "$X"

X=$((10 / 3))
check_eq "arith div" "3" "$X"

X=$((10 % 3))
check_eq "arith mod" "1" "$X"

# Backticks (legacy)
VAR=`echo "backtick"`
check_eq "backtick subst" "backtick" "$VAR"

echo ""
fi

#-----------------------------------------------------------------------------
if run_phase 13; then
echo "[Phase 13] Find"
echo "----------------------------------------"

mkdir -p finddir/sub1/deep
mkdir -p finddir/sub2
touch finddir/a.txt
touch finddir/sub1/b.txt
touch finddir/sub1/deep/c.txt
touch finddir/sub2/d.txt

OUT=$(find finddir -name "*.txt" 2>/dev/null | sort)
EXPECTED="finddir/a.txt
finddir/sub1/b.txt
finddir/sub1/deep/c.txt
finddir/sub2/d.txt"
check_eq "find -name" "$EXPECTED" "$OUT"

OUT=$(find finddir -type d 2>/dev/null | sort)
check_contains "find -type d" "finddir/sub1" "$OUT"

echo ""
fi

#=============================================================================
# Cleanup - remove test directory
cd /root
rm -rf coretest 2>/dev/null

#=============================================================================
echo "========================================"
echo " TEST RESULTS"
echo "========================================"
echo ""
TOTAL=$((PASS + FAIL))
echo "  Passed:  $PASS"
echo "  Failed:  $FAIL"
echo "  Total:   $TOTAL"
echo ""
if [ "$FAIL" -eq 0 ]; then
    echo "  Status:  ALL TESTS PASSED"
    echo ""
    echo "========================================"
    exit 0
else
    echo "  Status:  $FAIL TESTS FAILED"
    echo ""
    echo "========================================"
    exit 1
fi
"#;
    std::fs::write(root.join("root/test-core.sh"), test_core)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(root.join("root/test-core.sh"))?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(root.join("root/test-core.sh"), perms)?;
    }

    // Show what we created
    let applet_count = super::busybox::applets().len();
    println!("  📦 BusyBox binary + {} applet symlinks", applet_count);
    println!("  📄 /etc/inittab, passwd, group, profile");

    // Create CPIO archive
    let cpio_filename = format!("initramfs_{}.cpio", arch);
    let cpio_file = std::fs::File::create(&cpio_filename)?;
    
    let find = Command::new("find")
        .current_dir(&root)
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run find")?;

    let mut cpio = Command::new("cpio")
        .current_dir(&root)
        .args(["-o", "-H", "newc"])
        .stdin(find.stdout.unwrap())
        .stdout(cpio_file)
        .spawn()
        .context("Failed to run cpio")?;

    let status = cpio.wait()?;
    if !status.success() {
        bail!("cpio failed");
    }

    // Show final size
    let metadata = std::fs::metadata(&cpio_filename)?;
    let size_kb = metadata.len() / 1024;
    println!("✅ BusyBox initramfs created: {} ({} KB)", cpio_filename, size_kb);

    Ok(())
}

/// TEAM_435: Create test-specific initramfs with coreutils.
/// TEAM_438: Uses apps registry for external apps.
/// Includes init, shell, and required apps for testing.
pub fn create_test_initramfs(arch: &str) -> Result<()> {
    println!("Creating test initramfs for {}...", arch);
    let root = PathBuf::from("initrd_test_root");

    // Clean and create directory
    if root.exists() {
        std::fs::remove_dir_all(&root)?;
    }
    std::fs::create_dir(&root)?;

    let bare_target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    // Copy init and shell for boot
    let init_src = PathBuf::from(format!("crates/userspace/target/{}/release/init", bare_target));
    let shell_src = PathBuf::from(format!("crates/userspace/target/{}/release/shell", bare_target));

    if init_src.exists() {
        std::fs::copy(&init_src, root.join("init"))?;
    }
    if shell_src.exists() {
        std::fs::copy(&shell_src, root.join("shell"))?;
    }

    // Create hello.txt for cat test
    std::fs::write(root.join("hello.txt"), "Hello from initramfs!\n")?;

    // TEAM_438: Use apps registry - only include required apps for test initramfs
    let mut app_count = 0;
    for app in super::apps::required_apps() {
        let src = app.require(arch)?;
        std::fs::copy(&src, root.join(app.binary))?;
        app_count += 1;

        // Create symlinks for multi-call binaries
        for symlink_name in app.symlinks {
            #[cfg(unix)]
            {
                use std::os::unix::fs::symlink;
                let link_path = root.join(symlink_name);
                let _ = std::fs::remove_file(&link_path);
                symlink(app.binary, &link_path)?;
            }
            #[cfg(not(unix))]
            {
                std::fs::copy(&src, root.join(symlink_name))?;
            }
        }
    }

    println!("📦 Test initramfs: {} apps + init/shell", app_count);

    // Create CPIO archive
    let cpio_file = std::fs::File::create("initramfs_test.cpio")?;
    
    let find = Command::new("find")
        .current_dir(&root)
        .arg(".")
        .stdout(Stdio::piped())
        .spawn()
        .context("Failed to run find")?;

    let mut cpio = Command::new("cpio")
        .current_dir(&root)
        .args(["-o", "-H", "newc"])
        .stdin(find.stdout.unwrap())
        .stdout(cpio_file)
        .spawn()
        .context("Failed to run cpio")?;

    let status = cpio.wait()?;
    if !status.success() {
        bail!("cpio failed");
    }

    println!("✅ Created initramfs_test.cpio");
    Ok(())
}

fn build_kernel_with_features(features: &[&str], arch: &str) -> Result<()> {
    println!("Building kernel for {}...", arch);
    let target = match arch {
        "aarch64" => "aarch64-unknown-none",
        "x86_64" => "x86_64-unknown-none",
        _ => bail!("Unsupported architecture: {}", arch),
    };

    let mut args = vec![
        "build".to_string(),
        "-Z".to_string(), "build-std=core,alloc".to_string(),
        "--release".to_string(),
        "--target".to_string(), target.to_string(),
        "-p".to_string(), "levitate-kernel".to_string(),  // TEAM_426: Only build kernel, not all workspace members
    ];

    if !features.is_empty() {
        args.push("--features".to_string());
        args.push(features.join(","));
    }

    // Kernel is its own workspace - build from kernel directory
    let status = Command::new("cargo")
        .current_dir("crates/kernel")
        .args(&args)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Kernel build failed");
    }

    // Convert to binary for boot protocol support (Rule 38)
    if arch == "aarch64" {
        println!("Converting to raw binary...");
        let objcopy_status = Command::new("aarch64-linux-gnu-objcopy")
            .args([
                "-O", "binary",
                "crates/kernel/target/aarch64-unknown-none/release/levitate-kernel",
                "kernel64_rust.bin",
            ])
            .status()
            .context("Failed to run objcopy - is aarch64-linux-gnu-objcopy installed?")?;

        if !objcopy_status.success() {
            bail!("objcopy failed");
        }
    } else {
        // x86_64 uses multiboot2 (ELF) directly or needs different conversion
        println!("x86_64 kernel build complete (ELF format for multiboot2)");
    }

    Ok(())
}

/// TEAM_283: Build a bootable Limine ISO
// TEAM_435: Replaced Eyra with c-gull sysroot
// TEAM_444: Migrated to musl
pub fn build_iso(arch: &str) -> Result<()> {
    build_iso_internal(&[], arch, false)
}

/// TEAM_286: Build ISO with verbose feature for behavior testing
pub fn build_iso_verbose(arch: &str) -> Result<()> {
    build_iso_internal(&["verbose"], arch, false)
}

/// TEAM_374: Build ISO for testing with test initramfs
pub fn build_iso_test(arch: &str) -> Result<()> {
    build_iso_internal(&["verbose"], arch, true)
}

fn build_iso_internal(features: &[&str], arch: &str, use_test_initramfs: bool) -> Result<()> {
    if arch != "x86_64" {
        bail!("ISO build currently only supported for x86_64");
    }

    println!("💿 Building Limine ISO for {}...", arch);

    // TEAM_438: Build sysroot and all external apps if not present
    // TEAM_444: Now just ensures musl target is installed
    super::sysroot::ensure_rust_musl_target(arch)?;
    super::apps::ensure_all_built(arch)?;

    build_userspace(arch)?;
    // TEAM_451: Always use BusyBox initramfs now
    create_busybox_initramfs(arch)?;
    crate::disk::install_userspace_to_disk(arch)?;
    build_kernel_with_features(features, arch)?;

    let iso_root = PathBuf::from("iso_root");
    let boot_dir = iso_root.join("boot");
    
    // Clean and create staging directory
    if iso_root.exists() {
        std::fs::remove_dir_all(&iso_root)?;
    }
    std::fs::create_dir_all(&boot_dir)?;

    // 2. Copy components to ISO root
    let kernel_path = "crates/kernel/target/x86_64-unknown-none/release/levitate-kernel";
    // TEAM_374: Use test initramfs when in test mode
    let initramfs_path = if use_test_initramfs {
        "initramfs_test.cpio".to_string()
    } else {
        format!("initramfs_{}.cpio", arch)
    };
    let limine_cfg_path = "limine.cfg";

    std::fs::copy(kernel_path, boot_dir.join("levitate-kernel"))
        .context("Failed to copy levitate-kernel to ISO boot dir")?;
    if std::path::Path::new(&initramfs_path).exists() {
        std::fs::copy(&initramfs_path, boot_dir.join("initramfs.cpio"))
            .context("Failed to copy initramfs to ISO boot dir")?;
    }
    std::fs::copy(limine_cfg_path, iso_root.join("limine.cfg"))
        .context("Failed to copy limine.cfg - ensure it exists in repo root")?;

    // 3. Download/Prepare Limine binaries if needed
    prepare_limine_binaries(&iso_root)?;

    // 4. Create ISO using xorriso
    let iso_file = "levitate.iso";
    let status = Command::new("xorriso")
        .args([
            "-as", "mkisofs",
            "-b", "limine-bios-cd.bin",
            "-no-emul-boot", "-boot-load-size", "4", "-boot-info-table",
            "--efi-boot", "limine-uefi-cd.bin",
            "-efi-boot-part", "--efi-boot-image", "--protective-msdos-label",
            &iso_root.to_string_lossy(),
            "-o", iso_file,
        ])
        .status()
        .context("Failed to run xorriso")?;

    if !status.success() {
        bail!("xorriso failed to create ISO");
    }

    println!("✅ ISO created: {}", iso_file);
    Ok(())
}

// TEAM_435: build_eyra() removed - replaced by build::external::build_coreutils()

fn prepare_limine_binaries(iso_root: &PathBuf) -> Result<()> {
    let limine_dir = PathBuf::from("limine-bin");
    let files = [
        "limine-bios-cd.bin",
        "limine-uefi-cd.bin",
        "limine-bios.sys",
    ];
    
    // TEAM_304: Check if all required files exist, not just directory
    let all_files_exist = files.iter().all(|f| limine_dir.join(f).exists());
    
    if !all_files_exist {
        println!("📥 Downloading Limine binaries (v7.x)...");
        std::fs::create_dir_all(&limine_dir)?;
        
        let base_url = "https://github.com/limine-bootloader/limine/raw/v7.x-binary/";

        for file in &files {
            let url = format!("{}{}", base_url, file);
            let output = limine_dir.join(file);
            println!("  Fetching {}...", file);
            
            let status = Command::new("curl")
                .args(["-L", "-f", "-o", output.to_str().unwrap(), &url])
                .status()
                .context(format!("Failed to run curl for {}", file))?;
            
            if !status.success() {
                bail!("Failed to download {} from {}", file, url);
            }
        }
    }

    // Copy to ISO root for xorriso
    for file in &files {
        let src = limine_dir.join(file);
        let dst = iso_root.join(file);
        std::fs::copy(&src, &dst)
            .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))?;
    }

    Ok(())
}
