//! Text processing tests.
//!
//! Can users search, filter, and transform text?
//!
//! ## Anti-Reward-Hacking Design
//!
//! Each test chains all operations in a single exec() call to verify
//! the complete workflow functions correctly end-to-end.

use super::{test_result, Test, TestResult};
use crate::container::Container;

/// Test: Search text with grep
struct GrepSearch;

impl Test for GrepSearch {
    fn name(&self) -> &str { "grep search" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can search for patterns in files"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'line one\nerror: something broke\nline three\n' > /tmp/log.txt &&
                grep 'error' /tmp/log.txt &&
                rm /tmp/log.txt
            "#)?;

            if !result.contains("something broke") {
                anyhow::bail!("grep didn't find the error line: {}", result);
            }
            Ok("grep pattern matching works".into())
        })
    }
}

/// Test: Stream editing with sed
struct SedEdit;

impl Test for SedEdit {
    fn name(&self) -> &str { "sed substitution" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can perform find-and-replace on text"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                echo 'hello world' > /tmp/sed.txt &&
                sed 's/world/universe/' /tmp/sed.txt &&
                rm /tmp/sed.txt
            "#)?;

            if !result.contains("hello universe") {
                anyhow::bail!("sed substitution failed: {}", result);
            }
            Ok("sed s/// works correctly".into())
        })
    }
}

/// Test: Field extraction with awk
struct AwkFields;

impl Test for AwkFields {
    fn name(&self) -> &str { "awk fields" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can extract columns/fields from structured text"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'alice 100\nbob 200\ncharlie 300\n' > /tmp/data.txt &&
                awk '{print $2}' /tmp/data.txt &&
                rm /tmp/data.txt
            "#)?;

            if !result.contains("100") || !result.contains("200") || !result.contains("300") {
                anyhow::bail!("awk field extraction failed: {}", result);
            }
            Ok("awk field extraction works".into())
        })
    }
}

/// Test: Sort text
struct SortText;

impl Test for SortText {
    fn name(&self) -> &str { "sort" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can sort lines of text"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'zebra\napple\nmango\n' > /tmp/unsorted.txt &&
                sort /tmp/unsorted.txt &&
                rm /tmp/unsorted.txt
            "#)?;

            let lines: Vec<&str> = result.trim().lines().collect();
            if lines != vec!["apple", "mango", "zebra"] {
                anyhow::bail!("sort order wrong: {:?}", lines);
            }
            Ok("sort works correctly".into())
        })
    }
}

/// Test: Count with wc
struct WordCount;

impl Test for WordCount {
    fn name(&self) -> &str { "word count" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can count lines, words, and characters"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'one two three\nfour five\n' > /tmp/wc.txt &&
                wc -l < /tmp/wc.txt &&
                rm /tmp/wc.txt
            "#)?;

            if result.trim() != "2" {
                anyhow::bail!("Expected 2 lines, got {}", result.trim());
            }
            Ok("wc works correctly".into())
        })
    }
}

/// Test: Head and tail
struct HeadTail;

impl Test for HeadTail {
    fn name(&self) -> &str { "head/tail" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can view beginning or end of files"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'line1\nline2\nline3\nline4\nline5\n' > /tmp/lines.txt &&
                echo "HEAD:" && head -2 /tmp/lines.txt &&
                echo "TAIL:" && tail -2 /tmp/lines.txt &&
                rm /tmp/lines.txt
            "#)?;

            // Verify head gets first 2 lines
            if !result.contains("line1") || !result.contains("line2") {
                anyhow::bail!("head missing expected lines");
            }
            // Verify tail gets last 2 lines
            if !result.contains("line4") || !result.contains("line5") {
                anyhow::bail!("tail missing expected lines");
            }
            Ok("head and tail work correctly".into())
        })
    }
}

/// Test: Pipes work
struct Pipes;

impl Test for Pipes {
    fn name(&self) -> &str { "pipes" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can chain commands with pipes"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                printf 'ERROR: fail\nINFO: ok\nERROR: bad\n' > /tmp/pipe.txt &&
                grep ERROR /tmp/pipe.txt | wc -l &&
                rm /tmp/pipe.txt
            "#)?;

            if result.trim() != "2" {
                anyhow::bail!("pipe chain failed, expected 2 errors, got {}", result.trim());
            }
            Ok("command piping works".into())
        })
    }
}

/// Test: Compression with tar+gzip
struct Compression;

impl Test for Compression {
    fn name(&self) -> &str { "tar/gzip" }
    fn category(&self) -> &str { "text" }
    fn ensures(&self) -> &str {
        "User can create and extract compressed archives"
    }

    fn run(&self, c: &Container) -> TestResult {
        test_result(self.name(), self.ensures(), || {
            let result = c.exec_ok(r#"
                mkdir -p /tmp/archive &&
                echo 'file1' > /tmp/archive/a.txt &&
                echo 'file2' > /tmp/archive/b.txt &&
                tar -czf /tmp/archive.tar.gz -C /tmp archive &&
                rm -rf /tmp/archive &&
                tar -xzf /tmp/archive.tar.gz -C /tmp &&
                cat /tmp/archive/a.txt &&
                rm -rf /tmp/archive /tmp/archive.tar.gz
            "#)?;

            if !result.contains("file1") {
                anyhow::bail!("Archive extraction failed: {}", result);
            }
            Ok("tar/gzip compression works".into())
        })
    }
}

pub fn tests() -> Vec<Box<dyn Test>> {
    vec![
        Box::new(GrepSearch),
        Box::new(SedEdit),
        Box::new(AwkFields),
        Box::new(SortText),
        Box::new(WordCount),
        Box::new(HeadTail),
        Box::new(Pipes),
        Box::new(Compression),
    ]
}
