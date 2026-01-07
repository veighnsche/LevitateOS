# Questions: Crate Reorganization

**Team:** TEAM_101 (created by TEAM_103)  
**Date:** 2026-01-05  
**Related Plan:** `docs/planning/crate-reorganization/`

---

## Q1: Filesystem Crate Scope

Should `levitate-fs` wrap both FAT32 and ext4, or use separate crates?

**Options:**
- **A) Single crate** with feature flags for each filesystem
- **B) Separate crates** — `levitate-fs-fat32`, `levitate-fs-ext4`
- **C) Single crate** with unified trait, both impls always included

**Recommendation:** Option C — simpler, and kernel already uses both

**User Answer:** C

---

## Q2: Driver Trait Interface

Should drivers use a trait-based interface for testing?

**Options:**
- **A) Yes** — define `BlockDevice`, `NetDevice`, `InputDevice` traits
- **B) No** — concrete structs with impl, testing via integration tests only

**Recommendation:** Option A — enables mock drivers for unit testing

**User Answer:** A

---

## Q3: virtio-drivers Transition Strategy

How to handle the `virtio-drivers` dependency during transition?

**Options:**
- **A) Cold turkey** — Remove all virtio-drivers usage at once after levitate-virtio is fixed
- **B) Gradual** — Keep virtio-drivers for working drivers, replace one at a time
- **C) Parallel** — Keep both paths, feature-flag selection

**Recommendation:** Option B — safer, can verify each driver works before removing dependency

**User Answer:** A

---

## Blocking Status

These questions **do not block** Phase 2 Steps 1-3 (VirtQueue fix, HAL move, GPU rename).

They **may block** Phase 2 Steps 4-7 (driver extractions) depending on answers.

