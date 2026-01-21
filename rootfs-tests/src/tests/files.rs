//! File management tests.
//!
//! Can users create, copy, move, and manage files?
//!
//! ## Anti-Reward-Hacking Design
//!
//! Each test chains commands with && to verify the END-TO-END flow works.
//! This prevents false positives where individual commands succeed but
//! the overall capability is broken (e.g., files don't persist).

use super::{test_result, Test, TestResult};
use crate::container::Container;

/// Test: Create a file
struct CreateFile;

impl Test for CreateFile {
    fn name(&self) -> &str { "create file" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can create new files"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // Chain: create file, verify content, clean up
            // All in one exec to ensure state persists
            let result = c.exec_ok(r#"
                echo 'hello world' > /tmp/test_create.txt &&
                cat /tmp/test_create.txt &&
                rm /tmp/test_create.txt
            "#)?;

            if !result.contains("hello world") {
                anyhow::bail!("File content mismatch: {}", result);
            }
            Ok("Created and verified file content".into())
        })
    }
}

/// Test: Copy files
struct CopyFile;

impl Test for CopyFile {
    fn name(&self) -> &str { "copy file" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can copy files to new locations"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                echo 'original content' > /tmp/orig.txt &&
                cp /tmp/orig.txt /tmp/copy.txt &&
                cat /tmp/copy.txt &&
                rm /tmp/orig.txt /tmp/copy.txt
            "#)?;

            if !result.contains("original content") {
                anyhow::bail!("Copy content mismatch: {}", result);
            }
            Ok("cp preserves file content".into())
        })
    }
}

/// Test: Move/rename files
struct MoveFile;

impl Test for MoveFile {
    fn name(&self) -> &str { "move file" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can move and rename files"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // Move, verify original gone, verify new has content
            let result = c.exec_ok(r#"
                echo 'moveme' > /tmp/before.txt &&
                mv /tmp/before.txt /tmp/after.txt &&
                test ! -f /tmp/before.txt &&
                cat /tmp/after.txt &&
                rm /tmp/after.txt
            "#)?;

            if !result.contains("moveme") {
                anyhow::bail!("Moved file content wrong: {}", result);
            }
            Ok("mv removes original and preserves content".into())
        })
    }
}

/// Test: Delete files
struct DeleteFile;

impl Test for DeleteFile {
    fn name(&self) -> &str { "delete file" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can delete files"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            // Create, delete, verify gone (test ! -f fails if file exists)
            c.exec_ok(r#"
                echo 'delete me' > /tmp/todelete.txt &&
                rm /tmp/todelete.txt &&
                test ! -f /tmp/todelete.txt
            "#)?;

            Ok("rm completely removes files".into())
        })
    }
}

/// Test: Create directories
struct CreateDirectory;

impl Test for CreateDirectory {
    fn name(&self) -> &str { "create directory" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can create directory structures"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            c.exec_ok(r#"
                mkdir -p /tmp/testdir/subdir/deep &&
                test -d /tmp/testdir/subdir/deep &&
                rm -rf /tmp/testdir
            "#)?;

            Ok("mkdir -p creates nested directories".into())
        })
    }
}

/// Test: File permissions
struct FilePermissions;

impl Test for FilePermissions {
    fn name(&self) -> &str { "file permissions" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can change file permissions"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                echo '#!/bin/bash' > /tmp/script.sh &&
                chmod 755 /tmp/script.sh &&
                stat -c '%a' /tmp/script.sh &&
                rm /tmp/script.sh
            "#)?;

            if result.trim() != "755" {
                anyhow::bail!("Expected 755, got '{}'", result.trim());
            }
            Ok("chmod changes permissions correctly".into())
        })
    }
}

/// Test: Symbolic links
struct SymbolicLinks;

impl Test for SymbolicLinks {
    fn name(&self) -> &str { "symbolic links" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can create and follow symbolic links"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                echo 'target content' > /tmp/linktarget.txt &&
                ln -s /tmp/linktarget.txt /tmp/symlink.txt &&
                cat /tmp/symlink.txt &&
                readlink /tmp/symlink.txt &&
                rm /tmp/symlink.txt /tmp/linktarget.txt
            "#)?;

            if !result.contains("target content") {
                anyhow::bail!("Symlink doesn't resolve correctly");
            }
            if !result.contains("linktarget.txt") {
                anyhow::bail!("readlink doesn't show target");
            }
            Ok("symlinks resolve and readlink works".into())
        })
    }
}

/// Test: Find files
struct FindFiles;

impl Test for FindFiles {
    fn name(&self) -> &str { "find files" }
    fn category(&self) -> &str { "files" }
    fn ensures(&self) -> &str {
        "User can search for files by name or pattern"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                mkdir -p /tmp/findtest &&
                touch /tmp/findtest/file1.txt /tmp/findtest/file2.txt /tmp/findtest/other.log &&
                find /tmp/findtest -name '*.txt' | sort &&
                rm -rf /tmp/findtest
            "#)?;

            if !result.contains("file1.txt") || !result.contains("file2.txt") {
                anyhow::bail!("find didn't locate .txt files");
            }
            if result.contains("other.log") {
                anyhow::bail!("find incorrectly matched .log file");
            }
            Ok("find filters by pattern correctly".into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(CreateFile),
        Box::new(CopyFile),
        Box::new(MoveFile),
        Box::new(DeleteFile),
        Box::new(CreateDirectory),
        Box::new(FilePermissions),
        Box::new(SymbolicLinks),
        Box::new(FindFiles),
    ]
}
