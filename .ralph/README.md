# Ralph Loop — User Manual

## Quick Start (Sleep Mode)

```bash
cd ~/Projects/LevitateOS
tmux new -s ralph '.ralph/ralph.sh'
```

That's it. Detach with `Ctrl+B, D`. Go to sleep. Both AcornOS and IuppiterOS build in one interleaved loop.

## Reconnect

```bash
tmux attach -t ralph
```

Live output picks up where you left off. Scroll up with `Ctrl+B, [`.

## Commands

```bash
.ralph/ralph.sh              # Default: 50 iterations
.ralph/ralph.sh 30           # 30 iterations
.ralph/ralph.sh build 30     # Same (explicit)
```

Tasks are interleaved by dependency, not by variant. The PRD tags each task `[acorn]`, `[iuppiter]`, or `[shared]` — haiku picks the next unchecked task regardless of which variant it belongs to.

## What Happens

```
Iteration 1  [haiku $1]   — picks first unchecked PRD task, implements it, commits
Iteration 2  [haiku $1]   — next task
Iteration 3  [haiku $1]   — next task
   REVIEW    [opus  $5]   — reads last 3 commits, fixes bugs, does NOT pick up new tasks
Iteration 4  [haiku $1]   — next task
...repeat...
```

Each iteration:
1. Reads the PRD, progress file, and learnings
2. Implements ONE task (haiku) or reviews last 3 (opus)
3. Runs `cargo check` to verify
4. Commits inside the submodule
5. Marks the PRD task as done
6. Appends to progress and learnings files
7. Creates a team file in `.teams/`

## Stops When

- All PRD tasks are `[x]` and tests pass (outputs `<promise>COMPLETE</promise>`)
- Max iterations reached (default: 50)
- 3 consecutive iterations with no progress (stagnation)
- You press `Ctrl+C` (graceful cleanup, restores CLAUDE.md)

## Logs

```
.ralph/logs/build-001.log        # Full haiku output, iteration 1
.ralph/logs/build-002.log        # Iteration 2
.ralph/logs/review-003.log       # Opus review after iteration 3
.ralph/logs/modifications.log    # Soft warnings (tools/testing changes)
```

## Progress Files

```
.ralph/prd.md                    # Consolidated task checklist ([ ] → [x])
.ralph/progress.txt              # What each iteration did
.ralph/learnings.txt             # Patterns and gotchas discovered
```

## Team Files

Each iteration creates a team file in `.teams/TEAM_NNN_description.md` documenting what was implemented, decisions made, and any issues found.

## Safety

### Anti-reward-hack guard

After each iteration, the script checks that haiku didn't cheat by modifying test code instead of fixing the actual code.

**Hard block (reverted automatically):**
- `leviso/` — never touched
- `distro-spec/src/levitate/` — never touched
- `testing/cheat-guard/` — the anti-cheat macros
- `testing/install-tests/src/steps/` — test assertions
- `testing/install-tests/src/preflight.rs` — preflight checks

**Soft warn (logged, not reverted):**
- Other `testing/install-tests/` files (adding `--distro iuppiter` is legitimate)
- `tools/` (fixing real bugs is legitimate)

### Pre-commit hooks

Installed in AcornOS, IuppiterOS, distro-spec, distro-builder:
- `cargo fmt` auto-fixes formatting
- `cargo check` blocks broken commits
- Commit message must match `feat(acorn): ...` or `fix(iuppiter): ...`

### Rate limits

If Claude hits a rate limit, the script waits 5 minutes and retries (up to 5 times per iteration). No iterations wasted.

## Known Issues

### Install-Tests Boot Detection (TEAM_154)

The automated install-tests runner fails during initial boot detection. This is a Console I/O buffering issue in the test harness, not an ISO problem. Manual QEMU boot works perfectly. Phase 6 (post-reboot verification) has also been broken for ages.

See `.teams/TEAM_154_install-tests-broken-boot-detection.md` for details.

## If Things Go Wrong

### Reset to before the ralph loop ran

```bash
cd ~/Projects/LevitateOS
git reset --hard 6ed3d13
```

See `.ralph/RESET.md` for more reset options.

### Resume after abort

If the loop aborted due to stagnation at iteration 20:

```bash
.ralph/ralph.sh 30    # 30 more iterations from where it stopped
```

The PRD and progress files persist, so haiku picks up where it left off.

### Kill a stuck iteration

If tmux shows haiku is stuck (no output for minutes), press `Ctrl+C` once. The trap handler cleans up and restores CLAUDE.md. You can restart immediately.

## Cost Estimate

| Model | Per iteration | 50 iterations |
|-------|--------------|---------------|
| Haiku | ~$0.01-0.10 | ~$0.50-5.00 |
| Opus review | ~$1-3 | ~$16-48 (16 reviews) |
| **Total run** | | **~$5-60** |

## Config

Edit the top of `.ralph/ralph.sh`:

```bash
MODEL="haiku"              # Worker model
REVIEW_MODEL="opus"        # Reviewer model
REVIEW_EVERY=3             # Opus review frequency
REVIEW_BUDGET=5.00         # USD cap per opus review
DEFAULT_MAX_ITERATIONS=50  # Iterations per run
ITERATION_TIMEOUT=3600     # 60 min safety net per iteration
COOLDOWN_SECONDS=5         # Pause between iterations
MAX_BUDGET_PER_ITERATION=1.00  # USD cap per haiku iteration
MAX_STAGNANT_ITERATIONS=3  # Abort after N stuck iterations
```

## Summary Dashboard

When the loop finishes (or you `Ctrl+C`), you get:

```
═══ Ralph Loop Summary ═══
  Total time:          05h:23m:17s
  Tasks completed:     34
  Failed iterations:   2
  Timed out:           0
  Opus reviews:        11
  Rate limit waits:    1
  Reward hacks blocked: 0
  Logs:                .ralph/logs/
```
