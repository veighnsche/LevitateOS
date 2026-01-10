# Eyra Shell with Scripting Support

**TEAM_379** | Created: 2026-01-10 | **Reviewed: TEAM_391**

## ✅ Prerequisite: epoll Syscalls

brush uses **tokio** async runtime which requires epoll syscalls.

**Resolution**: Implement epoll syscalls in kernel (Phase 0) before brush integration.

**See:** [Questions](../../questions/TEAM_391_brush_shell_questions.md) for details.

## Problem Summary

The current shell (`lsh`) is no_std and lacks:
1. **Script execution** — Cannot run `.sh` files
2. **Variables** — No `$VAR` support
3. **Control flow** — No if/for/while
4. **Modern features** — No tab completion, history, etc.

This blocks testing and usability improvements.

## Proposed Solution

**Replace `lsh` with brush** — a POSIX/Bash compatible shell in Rust.

### Why brush?

| Feature | lsh (current) | brush |
|---------|---------------|-------|
| Language | no_std Rust | std Rust |
| **POSIX compatible** | ❌ | ✅ |
| **Bash compatible** | ❌ | ✅ |
| Variables | ❌ | ✅ `$var` |
| Arrays | ❌ | ✅ `${arr[@]}` |
| Loops | ❌ | ✅ for/while |
| Conditionals | ❌ | ✅ if/then/fi |
| Functions | ❌ | ✅ |
| Script files (.sh) | ❌ | ✅ |
| Tab completion | ❌ | ✅ (bash-completion) |
| History | ❌ | ✅ |
| Line editor | Basic | **reedline** |
| Test coverage | None | 900+ test cases |

### brush Credits (already uses these):
- **reedline** — Line editor (from Nushell team)
- **clap** — Command-line parsing
- **tokio** — Async runtime
- **nix** — POSIX system APIs

**Source:** https://github.com/reubeno/brush

## Phases

| Phase | Description | Status |
|-------|-------------|--------|
| [Phase 0](phase-0.md) | **Prerequisite** — Implement epoll syscalls in kernel | Pending |
| [Phase 1](phase-1.md) | Discovery — Analyze brush shell & requirements | Pending |
| [Phase 2](phase-2.md) | Design — Eyra adaptation strategy | Pending |
| [Phase 3](phase-3.md) | Implementation — Port brush to Eyra | Pending |
| [Phase 4](phase-4.md) | Integration & Testing | Pending |
| [Phase 5](phase-5.md) | Polish & Documentation | Pending |

## Success Criteria

- [ ] Shell runs on LevitateOS via Eyra
- [ ] Can execute script files (`bash script.sh` or `./script.sh`)
- [ ] Variables, loops, conditionals work (Bash syntax)
- [ ] Tab completion works
- [ ] History works
- [ ] All existing coreutils callable from shell
- [ ] Test scripts can verify coreutils behavior

## Design Decisions (Answered)

| Question | Answer |
|----------|--------|
| POSIX/Bash compatible? | ✅ **YES** — brush provides full compatibility |
| Syscall backend? | ✅ **Eyra std directly** — no shims |
| Line editor? | ✅ **reedline** — brush already uses it |
| Build integration? | ✅ **Eyra workspace** — like other coreutils |
| Binary name? | ✅ **`brush`** — use upstream name |
| Tokio/epoll resolution? | ✅ **Implement epoll** — Phase 0 prerequisite |

## References

- **brush:** https://github.com/reubeno/brush
- **reedline:** https://github.com/nushell/reedline
- **Bash reference:** https://www.gnu.org/software/bash/manual/
