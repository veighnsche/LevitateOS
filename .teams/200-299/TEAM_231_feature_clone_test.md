# TEAM_231_feature_clone_test

## Purpose
Create a userspace `clone_test` binary to verify the `sys_clone` thread creation implementation.

## Feature Summary
A standalone binary in `levbox` that exercises `sys_clone` to spawn a thread, verify memory sharing, and test thread joining via `CLONE_CHILD_CLEARTID` + futex.

## Status
- [ ] Phase 1: Discovery
- [ ] Phase 2: Design
- [ ] Phase 3: Implementation
- [ ] Phase 4: Integration
- [ ] Phase 5: Polish
