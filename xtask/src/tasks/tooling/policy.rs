use anyhow::{Context, Result};
use leviso_cheat_guard::{cheat_bail, cheat_ensure};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

const SCAN_ROOTS: &[&str] = &[
    "distro-builder",
    "distro-contract",
    "distro-variants",
    "testing/install-tests",
    "AcornOS",
    "IuppiterOS",
    "RalphOS",
    "xtask",
];

const SCAN_EXTS: &[&str] = &["rs", "sh", "toml", "rhai"];

#[derive(Clone, Copy)]
struct ForbiddenRegexSpec {
    pattern: &'static str,
    reason: &'static str,
    escalated: bool,
}

const FORBIDDEN_LINE_REGEX_SPECS: &[ForbiddenRegexSpec] = &[
    ForbiddenRegexSpec {
        pattern: r#"(?i)\b(?:leviso|acornos|iuppiteros|ralphos)/downloads/"#,
        reason: "legacy crate downloads path binding detected",
        escalated: false,
    },
    ForbiddenRegexSpec {
        pattern: r#"join\("downloads/\.tools"\)"#,
        reason: "tools path fallback to per-distro downloads detected",
        escalated: true,
    },
];

const FORBIDDEN_NORMALIZED_REGEX_SPECS: &[ForbiddenRegexSpec] = &[
    ForbiddenRegexSpec {
        pattern: r#"join\("(?:leviso|AcornOS|IuppiterOS|RalphOS)"\)\.join\("downloads"\)"#,
        reason: "legacy crate downloads join-chain detected",
        escalated: false,
    },
    ForbiddenRegexSpec {
        pattern: r#"read_dir\([^)]+\).+join\("downloads/\.tools"\)"#,
        reason: "dynamic downloads/.tools autodiscovery bypass detected",
        escalated: true,
    },
];

#[derive(Debug)]
struct Violation {
    path: PathBuf,
    line: usize,
    token: String,
    reason: String,
    content: String,
    escalated: bool,
}

const LEGACY_CRATE_NAMES: &[&str] = &["leviso", "AcornOS", "IuppiterOS", "RalphOS"];
const PASS_MARK: &str = "\x1b[1;32m[PASSED]\x1b[0m";
const FAIL_MARK: &str = "\x1b[1;31m[FAILED]\x1b[0m";

pub fn audit_legacy_bindings() -> Result<()> {
    let root = crate::util::repo::repo_root()?;
    let line_regexes = compile_regex_specs(FORBIDDEN_LINE_REGEX_SPECS)
        .context("compiling forbidden line regex specs")?;
    let normalized_regexes = compile_regex_specs(FORBIDDEN_NORMALIZED_REGEX_SPECS)
        .context("compiling forbidden normalized regex specs")?;
    cheat_ensure!(
        root.join("distro-variants").is_dir(),
        protects = "Policy observer runs against the correct repository root",
        severity = "CRITICAL",
        cheats = [
            "Run audit from a non-repo path so checks silently miss files",
            "Shadow repo root resolution and scan an empty tree",
        ],
        consequence = "Legacy bindings remain in code and builds/tests pass as false positives",
        "policy audit repo root '{}' does not contain expected 'distro-variants/'",
        root.display()
    );
    let mut violations = Vec::new();

    for rel in SCAN_ROOTS {
        let base = root.join(rel);
        if !base.exists() {
            continue;
        }
        collect_violations(
            &root,
            &base,
            &line_regexes,
            &normalized_regexes,
            &mut violations,
        )
        .with_context(|| format!("scanning '{}'", base.display()))?;
    }

    // Include root justfile explicitly.
    let justfile = root.join("justfile");
    if justfile.is_file() {
        scan_file(
            &root,
            &justfile,
            &line_regexes,
            &normalized_regexes,
            &mut violations,
        )
        .with_context(|| format!("scanning '{}'", justfile.display()))?;
    }

    if violations.is_empty() {
        println!(
            "policy audit {}: no forbidden legacy bindings found",
            PASS_MARK
        );
        return Ok(());
    }
    let escalated_count = violations.iter().filter(|v| v.escalated).count();

    eprintln!(
        "policy audit {}: found {} forbidden legacy binding(s):",
        FAIL_MARK,
        violations.len()
    );
    for v in &violations {
        eprintln!(
            "- {}:{} token='{}'\n  reason: {}\n  line: {}",
            v.path.display(),
            v.line,
            v.token,
            v.reason,
            v.content.trim()
        );
    }
    if escalated_count > 0 {
        cheat_bail!(
            protects =
                "Cheat guard integrity: dynamic fallback paths cannot bypass legacy-binding policy",
            severity = "CRITICAL (ESCALATED)",
            cheats = [
                "Auto-discover */downloads/.tools to avoid explicit legacy tokens",
                "Route tooling resolution through directory scans to evade token-based guards",
                "Reintroduce legacy coupling behind compatibility fallback behavior",
                "Preserve green checks while bypassing migration policy intent",
            ],
            consequence = "Policy enforcement is bypassed and build/test tooling silently re-couples to deprecated entrypoints",
            "escalated policy violation(s) detected ({}): dynamic legacy-bypass patterns are forbidden",
            escalated_count
        );
    }
    cheat_bail!(
        protects = "Stage wiring remains fully migrated to new entrypoints without legacy crate path coupling",
        severity = "CRITICAL",
        cheats = [
            "Rewire stage inputs to legacy */downloads paths and keep green checks",
            "Hide legacy path composition with split join() chains",
            "Move legacy path usage into less-reviewed tooling paths",
            "Rely on known-token-only scanners that miss novel legacy path variants",
        ],
        consequence = "Builds appear healthy while reintroducing legacy coupling, causing non-reproducible stage behavior and false migration progress",
        "forbidden legacy bindings detected ({} violation(s)); migrate stage wiring to non-legacy producers",
        violations.len()
    )
}

fn collect_violations(
    root: &Path,
    dir: &Path,
    line_regexes: &[(Regex, &'static str, &'static str, bool)],
    normalized_regexes: &[(Regex, &'static str, &'static str, bool)],
    violations: &mut Vec<Violation>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("reading '{}'", dir.display()))? {
        let entry = entry.with_context(|| format!("reading entry in '{}'", dir.display()))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("reading metadata '{}'", path.display()))?;

        if metadata.is_dir() {
            collect_violations(root, &path, line_regexes, normalized_regexes, violations)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        if !should_scan_file(&path, root) {
            continue;
        }

        scan_file(root, &path, line_regexes, normalized_regexes, violations)?;
    }

    Ok(())
}

fn should_scan_file(path: &Path, root: &Path) -> bool {
    let rel = relative_path(path, root);
    if rel.as_path() == Path::new("xtask/src/tasks/tooling/policy.rs") {
        return false;
    }

    if rel.as_path() == Path::new("justfile") {
        return true;
    }

    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SCAN_EXTS.contains(&ext))
        .unwrap_or(false)
}

fn scan_file(
    root: &Path,
    path: &Path,
    line_regexes: &[(Regex, &'static str, &'static str, bool)],
    normalized_regexes: &[(Regex, &'static str, &'static str, bool)],
    violations: &mut Vec<Violation>,
) -> Result<()> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };
    let rel = relative_path(path, root);

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let is_comment = trimmed.starts_with('#') || trimmed.starts_with("//");
        for (regex, pattern, reason, escalated) in line_regexes {
            if !regex.is_match(line) {
                continue;
            }
            if is_comment {
                continue;
            }
            violations.push(Violation {
                path: rel.clone(),
                line: idx + 1,
                token: (*pattern).to_string(),
                reason: (*reason).to_string(),
                content: line.to_string(),
                escalated: *escalated,
            });
        }

        if looks_like_legacy_join_chain_line(&rel, line) {
            violations.push(Violation {
                path: rel.clone(),
                line: idx + 1,
                token: "join(<legacy-crate>).join(\"downloads\")".to_string(),
                reason: "legacy crate downloads path composition detected".to_string(),
                content: line.to_string(),
                escalated: false,
            });
        }
    }

    // Catch split/whitespace-obfuscated join chains across lines.
    let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();
    for (regex, pattern, reason, escalated) in normalized_regexes {
        if regex.is_match(&normalized) {
            violations.push(Violation {
                path: rel.clone(),
                line: 1,
                token: (*pattern).to_string(),
                reason: (*reason).to_string(),
                content: "(multi-line join chain)".to_string(),
                escalated: *escalated,
            });
        }
    }
    Ok(())
}

fn looks_like_legacy_join_chain_line(path: &Path, line: &str) -> bool {
    if path == Path::new("distro-contract/src/runtime.rs") {
        return false;
    }
    let has_join_downloads = line.contains("join(\"downloads\")");
    if !has_join_downloads {
        return false;
    }
    LEGACY_CRATE_NAMES
        .iter()
        .any(|name| line.contains(&format!("join(\"{name}\")")))
}

fn relative_path(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root)
        .map(Path::to_path_buf)
        .unwrap_or_else(|_| path.to_path_buf())
}

fn compile_regex_specs(
    specs: &[ForbiddenRegexSpec],
) -> Result<Vec<(Regex, &'static str, &'static str, bool)>> {
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        let regex = Regex::new(spec.pattern)
            .with_context(|| format!("invalid regex pattern '{}'", spec.pattern))?;
        out.push((regex, spec.pattern, spec.reason, spec.escalated));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leviso_cheat_guard::cheat_aware;

    #[cheat_aware(
        protects = "Legacy downloads join chains are detected even when path text is not contiguous",
        severity = "CRITICAL",
        ease = "MEDIUM",
        cheats = [
            "Split legacy path into join() fragments to bypass plain string-token scans",
            "Move legacy path composition to a helper function to evade simple grep-based checks"
        ],
        consequence = "Policy observer misses legacy bindings and allows migration regressions"
    )]
    #[test]
    fn detects_multi_line_legacy_join_chain() {
        let content = r#"
            let p = root
                .join("leviso")
                .join("downloads")
                .join("rootfs");
        "#;
        let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(normalized.contains("join(\"leviso\").join(\"downloads\")"));
    }

    #[cheat_aware(
        protects = "Comment text does not trigger actionable policy failures",
        severity = "HIGH",
        ease = "EASY",
        cheats = [
            "Flood comments with known-bad tokens to normalize real violations",
            "Cause policy fatigue by generating non-actionable false positives"
        ],
        consequence = "Engineers ignore guard output and real legacy bindings slip through"
    )]
    #[test]
    fn ignores_comment_line_tokens() {
        let line = "# tools_prefix := join(justfile_directory(), \"leviso/downloads/.tools\")";
        let trimmed = line.trim_start();
        let is_comment = trimmed.starts_with('#') || trimmed.starts_with("//");
        assert!(is_comment);
        let (regex, _, _, _) = compile_regex_specs(FORBIDDEN_LINE_REGEX_SPECS)
            .expect("compile line regex")
            .into_iter()
            .next()
            .expect("line regex exists");
        assert!(regex.is_match(line), "fixture should match raw token");
    }

    #[cheat_aware(
        protects = "Dynamic scans for */downloads/.tools cannot bypass legacy policy by hiding crate names",
        severity = "CRITICAL",
        ease = "EASY",
        cheats = [
            "Replace explicit legacy crate token with read_dir(root) scan",
            "Return first discovered downloads/.tools candidate to keep legacy compatibility"
        ],
        consequence = "Build tooling silently falls back to deprecated distro trees and invalidates migration guarantees"
    )]
    #[test]
    fn detects_dynamic_downloads_tools_autodiscovery() {
        let content = r#"
            if let Ok(entries) = std::fs::read_dir(root) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join("downloads/.tools");
                    if candidate.is_dir() {
                        return candidate;
                    }
                }
            }
        "#;
        let normalized: String = content.chars().filter(|c| !c.is_whitespace()).collect();
        let specs = compile_regex_specs(FORBIDDEN_NORMALIZED_REGEX_SPECS)
            .expect("compile normalized regex");
        let matched = specs.iter().any(|(regex, _, reason, escalated)| {
            *escalated
                && *reason == "dynamic downloads/.tools autodiscovery bypass detected"
                && regex.is_match(&normalized)
        });
        assert!(matched, "dynamic autodiscovery bypass must be detected");
    }
}
