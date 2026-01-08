# TEAM_204: Proactive Tmpfs Bug Investigation

## Objective
Investigate and fix edge cases, hidden bugs, and missing functionalities in the `tmpfs` VFS integration refactoring.

## Timeline
- **Discovery**: [2026-01-06]
- **Team**: TEAM_204

## Progress
- [ ] Hypothesis 1: Rename Dir into Subdir Loop
- [ ] Hypothesis 2: getdents buffer overflow/alignment
- [ ] Hypothesis 3: Missing utimensat implementation (VFS level)
- [ ] Hypothesis 4: Missing readlinkat implementation
- [ ] Hypothesis 5: symlink loop detection in lookup
- [ ] Hypothesis 6: recursive locking/deadlock in complex rename

## Findings
(To be updated)
