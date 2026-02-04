# Ralph Loop Reset Points

Created: 2026-02-04, before first ralph loop run.

## Safe Reset (pre-ralph-loop baseline)

```bash
cd /home/vince/Projects/LevitateOS
git reset --hard a8a3e34
```

This commit includes:
- IuppiterOS submodule added and initialized
- .ralph/ directory with ralph.sh, PRDs, CLAUDE.md files
- All submodule pointers up to date (distro-spec, distro-builder, leviso, etc.)
- ACORNOS_MIRROING_CHECKLIST.md
- distro-builder executor extraction committed

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
