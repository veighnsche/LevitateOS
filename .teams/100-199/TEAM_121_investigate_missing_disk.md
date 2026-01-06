# TEAM_121: Investigate Missing tinyos_disk.img

## Symptom
`bash ./run.sh` (which calls `cargo xtask run`) fails with:
`qemu-system-aarch64: -drive file=tinyos_disk.img,format=raw,if=none,id=hd0: Could not open 'tinyos_disk.img': No such file or directory`

## Hypotheses
1. `cargo xtask run` assumes the disk image already exists.
2. The disk image was manually deleted (by me) to resolve a process lock, but the build system doesn't recreate it.

## Investigation Log
- [2026-01-05 22:15] Initialized investigation.
- [2026-01-05 22:18] Found that `terminal::write_str` calls `gpu_state.flush()` on every invocation.
- [2026-01-05 22:19] confirmed that `libsyscall` and `levitate-hal` use string formatting that results in multiple `write_str` calls for a single line of output.
- [2026-01-05 22:19] formed plan to remove immediate flush and rely on periodic updates.

## Root Cause
The kernel flushes the entire GPU framebuffer to the hardware (via VirtIO) on every console output chunk. Since formatting and shells often write 1-4 characters at a time, this creates a slow "character-by-character" visual effect and bottlenecks the system.
