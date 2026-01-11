# Phase 5: Hardening

## Test Coverage Goals

### Unit Tests per Crate

| Crate | Target Coverage | Key Areas |
|-------|-----------------|-----------|
| `mm` | 80% | Frame allocation, CoW, mapping |
| `sched` | 75% | Scheduler fairness, wait queues |
| `vfs` | 70% | Path resolution, mount table |
| `syscall` | 85% | All syscall paths |
| `arch/*` | 50% | Context switch, page tables |

### Test Infrastructure

Each crate should support:

```toml
# mm/Cargo.toml
[features]
std = []  # Enable for host-side testing

[dev-dependencies]
proptest = "1.0"  # Property-based testing
```

**Host-side Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_alloc_dealloc() {
        // Can run on host with mocked physical memory
    }
}
```

**Integration Tests** (in `tests/` directory):
```rust
// mm/tests/integration.rs
#![cfg(feature = "std")]

#[test]
fn test_cow_semantics() {
    // Full CoW test with simulated page faults
}
```

### Behavior Test Updates

Update golden files after refactor:

```bash
# Regenerate all golden files
cargo xtask test behavior --update
cargo xtask test debug --update
```

Verify no behavioral regression:
```bash
cargo xtask test behavior
cargo xtask test unit
```

## Static Analysis Gates

### Required Checks

All PRs must pass:

```yaml
# .github/workflows/ci.yml
- name: Clippy
  run: |
    cargo clippy --workspace --target x86_64-unknown-none -- -D warnings
    cargo clippy --workspace --target aarch64-unknown-none -- -D warnings

- name: Format
  run: cargo fmt --all -- --check

- name: Doc
  run: cargo doc --workspace --no-deps

- name: Miri (where applicable)
  run: cargo +nightly miri test -p los_mm --features std
```

### Unsafe Audit

After refactor, audit all `unsafe` blocks:

```bash
# Find all unsafe blocks
grep -rn "unsafe" --include="*.rs" | grep -v "// SAFETY:" > unsafe_audit.txt
```

Every `unsafe` block MUST have a `// SAFETY:` comment. Create tracking issue for any missing.

### Dependency Audit

```bash
cargo audit
cargo deny check
```

Add `deny.toml` for license and security checks:

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause"]
```

## Performance Verification

### Boot Time Benchmark

Measure before and after refactor:

```bash
# Before refactor
cargo xtask run --headless --timeout 5 2>&1 | grep "Boot complete"
# Record: Boot complete in XXXms

# After refactor
cargo xtask run --headless --timeout 5 2>&1 | grep "Boot complete"
# Compare: Boot complete in YYYms
```

**Acceptance Criteria**: < 10% regression in boot time.

### Memory Usage

Check kernel memory footprint:

```bash
# Size of kernel binary
ls -la target/*/release/levitate-kernel

# Before: XXX KB
# After: YYY KB
```

**Acceptance Criteria**: < 5% growth in kernel binary size.

### Context Switch Latency

If timing infrastructure exists:

```rust
// In scheduler test
let start = timer::now();
for _ in 0..1000 {
    task::yield_now();
}
let elapsed = timer::now() - start;
// Average: elapsed / 1000
```

## Documentation Hardening

### API Documentation

Every public item must have docs:

```rust
/// Allocate a physical frame.
///
/// # Returns
/// - `Ok(PhysFrame)` - Successfully allocated frame
/// - `Err(MmError::OutOfMemory)` - No frames available
///
/// # Example
/// ```
/// let frame = alloc_frame()?;
/// // Use frame...
/// free_frame(frame);
/// ```
pub fn alloc_frame() -> Result<PhysFrame, MmError> {
    // ...
}
```

### Architecture Decision Records

Create ADRs for major decisions in `docs/adr/`:

```markdown
# ADR-001: Process/Thread Separation

## Status
Accepted

## Context
Task struct was mixing process and thread concerns.

## Decision
Separate into Process (address space, fd table) and Thread (context, state).

## Consequences
- Clearer ownership
- Easier multi-threading support
- More code in scheduler crate
```

## Regression Prevention

### CI Gate Configuration

```yaml
# Branch protection rules
required_status_checks:
  - build-x86_64
  - build-aarch64
  - test-unit
  - test-behavior
  - clippy
  - fmt
```

### Golden File Lock

After Phase 5, mark all golden files as "gold" (strict):

```toml
# xtask.toml
[golden]
# All files are gold (strict) after refactor
default_rating = "gold"
```

### Commit Message Convention

Enforce conventional commits for refactor:

```
refactor(mm): extract memory management to los_mm crate

- Move src/memory/*.rs to mm/src/
- Update 47 call sites
- All tests pass

Part of TEAM_422 kernel architecture redesign.
```

## Completion Checklist

### Phase 5 Complete When:

- [ ] All crates have README.md
- [ ] All public APIs documented
- [ ] All `unsafe` blocks have `// SAFETY:` comments
- [ ] Unit tests exist for each crate
- [ ] All existing behavior tests pass
- [ ] Boot time within 10% of baseline
- [ ] Binary size within 5% of baseline
- [ ] CI pipeline enforces all checks
- [ ] Team file updated with final status
- [ ] ADRs written for major decisions

### Final Verification

```bash
# Full verification suite
cargo xtask check                      # Preflight checks
cargo build --target x86_64-unknown-none --release
cargo build --target aarch64-unknown-none --release
cargo xtask test unit
cargo xtask test behavior
cargo xtask test regress
cargo xtask run --headless --timeout 10  # Boot test
```

### Handoff

Update main CLAUDE.md with:
1. New workspace structure
2. Updated import patterns
3. New crate boundaries
4. Changed build commands (if any)

Archive old planning documents:
```bash
mv docs/planning/kernel-architecture-redesign docs/archive/TEAM_422/
```
