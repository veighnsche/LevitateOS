# Ralph Loop Reset Points

Created: 2026-02-04, before first ralph loop run.

## Safe Reset (pre-ralph-loop baseline)

```bash
cd /home/vince/Projects/LevitateOS
git reset --hard 6ed3d13
```

This commit includes everything below plus:
- cargo fmt across all 17 submodules (clean formatting baseline)
- Anti-reward-hack guard with baseline comparison
- Pre-commit hooks (cargo fmt + cargo check + commit message format)
- Rate limit retry logic

Previous safe reset (before fmt + hooks): `a8a3e34`

## Nuclear Reset (before any ralph/iuppiter work)

```bash
cd /home/vince/Projects/LevitateOS
git reset --hard 16d744a
```

This is the last commit before IuppiterOS submodule and .ralph/ were added.
You will lose: IuppiterOS submodule, .ralph/ directory, submodule pointer updates,
distro-builder executor extraction, and the mirroring checklist.

After nuclear reset, also clean up the IuppiterOS submodule remnants:
```bash
rm -rf IuppiterOS
git submodule deinit IuppiterOS 2>/dev/null
```

## Submodule commits at baseline

| Submodule | Commit |
|-----------|--------|
| distro-spec | 9f0fbcd (includes iuppiter variant skeleton) |
| distro-builder | 134d0ae (includes executor extraction) |
| IuppiterOS | 36dc27e (empty init) |

## What the ralph loop will touch

- `AcornOS/` submodule (builder implementation)
- `IuppiterOS/` submodule (builder implementation)
- `distro-spec/src/acorn/` (AcornOS specs, maybe)
- `distro-spec/src/iuppiter/` (IuppiterOS specs, maybe)
- `distro-builder/` (shared abstractions, maybe)
- `.ralph/*-progress.txt` and `.ralph/*-learnings.txt`
- `.ralph/acorn-prd.md` and `.ralph/iuppiter-prd.md` (task checkboxes)

It should NOT touch: `leviso/`, `distro-spec/src/levitate/`, test expectations.
