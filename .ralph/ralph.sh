#!/usr/bin/env bash
set -uo pipefail
# NOTE: no `set -e` — we handle errors explicitly throughout

# =============================================================================
# Ralph Loop — AcornOS + IuppiterOS Builder
# =============================================================================
#
# Runs Claude Code (haiku) in a loop to build AcornOS and IuppiterOS.
# Each iteration: read PRD + progress → implement one task → test → commit.
#
# Usage:
#   .ralph/ralph.sh acorn [max_iterations]    # Build AcornOS (default: 50)
#   .ralph/ralph.sh iuppiter [max_iterations] # Build IuppiterOS (default: 50)
#   .ralph/ralph.sh all [max_iterations]      # AcornOS first, then IuppiterOS
#
# Requirements:
#   - claude CLI installed and authenticated
#   - Running from LevitateOS project root
#   - ~6 hours unattended time
#   - Internet connection for Claude API

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RALPH_DIR="$SCRIPT_DIR"
LOG_DIR="$RALPH_DIR/logs"

# ── Config ───────────────────────────────────────────────────────────────────
MODEL="haiku"
REVIEW_MODEL="opus"
REVIEW_EVERY=3                 # run an opus review every N haiku iterations
REVIEW_BUDGET=5.00             # USD cap for opus review (more expensive, but targeted)
DEFAULT_MAX_ITERATIONS=50
ITERATION_TIMEOUT=3600         # 60 minutes max per iteration (install-tests need QEMU)
COOLDOWN_SECONDS=5             # pause between iterations (rate limit protection)
RATE_LIMIT_WAIT=300            # 5 minutes wait on rate limit before retry
MAX_RATE_LIMIT_RETRIES=5       # max retries per iteration on rate limit
MAX_BUDGET_PER_ITERATION=1.00  # USD cap per iteration (haiku is ~$0.01-0.10)
MAX_STAGNANT_ITERATIONS=3      # abort after N iterations with no progress change
# ─────────────────────────────────────────────────────────────────────────────

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

log()     { echo -e "${BLUE}[ralph]${NC} $*"; }
warn()    { echo -e "${YELLOW}[ralph]${NC} $*"; }
error()   { echo -e "${RED}[ralph]${NC} $*" >&2; }
success() { echo -e "${GREEN}[ralph]${NC} $*"; }
dim()     { echo -e "${DIM}$*${NC}"; }
header()  { echo -e "\n${BOLD}${CYAN}$*${NC}\n"; }

# Track total elapsed time
TOTAL_START_TIME=$SECONDS
COMPLETED_TASKS=0
FAILED_ITERATIONS=0
TIMED_OUT_ITERATIONS=0
RATE_LIMITED_WAITS=0
REWARD_HACKS_BLOCKED=0
REVIEW_ITERATIONS=0

# =============================================================================
# Utilities
# =============================================================================

elapsed() {
    local seconds=$1
    printf '%02dh:%02dm:%02ds' $((seconds/3600)) $((seconds%3600/60)) $((seconds%60))
}

md5_file() {
    md5sum "$1" 2>/dev/null | cut -d' ' -f1 || echo "empty"
}

count_checked() {
    local n
    n=$(grep -c '^- \[x\]' "$1" 2>/dev/null) || true
    echo "${n:-0}"
}

count_unchecked() {
    local n
    n=$(grep -c '^- \[ \]' "$1" 2>/dev/null) || true
    echo "${n:-0}"
}

# =============================================================================
# Preflight
# =============================================================================

preflight() {
    local errors=0

    if ! command -v claude &>/dev/null; then
        error "claude CLI not found"
        ((errors++))
    fi

    if ! command -v timeout &>/dev/null; then
        error "timeout command not found (coreutils)"
        ((errors++))
    fi

    if ! command -v tee &>/dev/null; then
        error "tee command not found"
        ((errors++))
    fi

    if [[ ! -d "$PROJECT_ROOT/.git" ]]; then
        error "Not in a git repo. Run from LevitateOS root."
        ((errors++))
    fi

    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        error "No Cargo.toml found. Are you in LevitateOS root?"
        ((errors++))
    fi

    if [[ $errors -gt 0 ]]; then
        error "$errors preflight check(s) failed"
        exit 1
    fi

    success "Preflight OK"
}

# =============================================================================
# Git hooks for writable submodules
# =============================================================================
# Installed before each phase. Gives haiku fast feedback:
#   pre-commit:  cargo fmt (auto-fix) + cargo check (block if broken)
#   commit-msg:  enforce feat(acorn)/fix(iuppiter) format

install_hooks() {
    local hook_src="$RALPH_DIR/hooks"
    [[ -d "$hook_src" ]] || return 0

    # Submodules haiku is allowed to commit in
    local writable_subs=(AcornOS IuppiterOS distro-spec distro-builder)

    for sub in "${writable_subs[@]}"; do
        local sub_path="$PROJECT_ROOT/$sub"
        [[ -d "$sub_path" ]] || continue

        # Find the hooks dir (submodules use .git/modules/X/hooks or .git/hooks)
        local hooks_dir
        if [[ -f "$sub_path/.git" ]]; then
            # Submodule with .git file pointing to parent's .git/modules/
            local gitdir
            gitdir=$(cat "$sub_path/.git" | sed 's/^gitdir: //')
            hooks_dir="$sub_path/$gitdir/hooks"
        elif [[ -d "$sub_path/.git" ]]; then
            hooks_dir="$sub_path/.git/hooks"
        else
            continue
        fi

        mkdir -p "$hooks_dir"
        cp "$hook_src/pre-commit" "$hooks_dir/pre-commit"
        cp "$hook_src/commit-msg" "$hooks_dir/commit-msg"
        chmod +x "$hooks_dir/pre-commit" "$hooks_dir/commit-msg"
    done

    log "Installed git hooks in writable submodules"
}

# =============================================================================
# CLAUDE.md management
# =============================================================================

install_claude_md() {
    local phase="$1"
    local source="$RALPH_DIR/CLAUDE-${phase}.md"

    if [[ ! -f "$source" ]]; then
        error "Missing $source"
        exit 1
    fi

    # Back up original if not already backed up
    if [[ -f "$PROJECT_ROOT/CLAUDE.md" && ! -f "$RALPH_DIR/.claude-md-backup" ]]; then
        cp "$PROJECT_ROOT/CLAUDE.md" "$RALPH_DIR/.claude-md-backup"
        log "Backed up CLAUDE.md"
    fi

    cp "$source" "$PROJECT_ROOT/CLAUDE.md"
    # Store checksum to detect if claude modifies it
    md5_file "$PROJECT_ROOT/CLAUDE.md" > "$RALPH_DIR/.claude-md-checksum"
    log "Installed CLAUDE.md for phase: $phase"
}

restore_claude_md() {
    if [[ -f "$RALPH_DIR/.claude-md-backup" ]]; then
        cp "$RALPH_DIR/.claude-md-backup" "$PROJECT_ROOT/CLAUDE.md"
        rm -f "$RALPH_DIR/.claude-md-backup" "$RALPH_DIR/.claude-md-checksum"
        log "Restored original CLAUDE.md"
    fi
}

verify_claude_md() {
    local phase="$1"
    local expected
    expected=$(cat "$RALPH_DIR/.claude-md-checksum" 2>/dev/null || echo "")
    local actual
    actual=$(md5_file "$PROJECT_ROOT/CLAUDE.md")

    if [[ -n "$expected" && "$expected" != "$actual" ]]; then
        warn "Claude modified CLAUDE.md — restoring phase context"
        install_claude_md "$phase"
    fi
}

# =============================================================================
# Anti-reward-hack guard
# =============================================================================
# Two tiers of protection:
#
# HARD BLOCK (revert + fail iteration):
#   - leviso/                        — never any reason to touch
#   - distro-spec/src/levitate/      — never any reason to touch
#   - testing/cheat-guard/           — gutting protection macros is cheating
#   - testing/install-tests/src/steps/ — changing what "pass" means is cheating
#
# SOFT WARN (log to file, don't revert):
#   - testing/install-tests/ (other files) — might be adding iuppiter distro context
#   - testing/rootfs-tests/               — might be legitimate
#   - tools/                              — might be fixing real bugs
#
# ALLOWED (no check):
#   - AcornOS/, IuppiterOS/          — the whole point
#   - distro-spec/src/acorn/         — acorn specs
#   - distro-spec/src/iuppiter/      — iuppiter specs
#   - distro-builder/                — shared abstractions

# Paths that get reverted unconditionally (relative to submodule root)
hard_block_submodule() {
    local sub="$1"
    local sub_path="$PROJECT_ROOT/$sub"
    [[ -d "$sub_path/.git" || -f "$sub_path/.git" ]] || return 0

    local dirty
    dirty=$(cd "$sub_path" && git status --porcelain 2>/dev/null)
    if [[ -n "$dirty" ]]; then
        error "HARD BLOCK: $sub modified — reverting"
        (cd "$sub_path" && git checkout -- . && git clean -fd) 2>/dev/null
        # Also reset submodule pointer in parent
        (cd "$PROJECT_ROOT" && git checkout -- "$sub") 2>/dev/null
        return 1
    fi

    local parent_diff
    parent_diff=$(cd "$PROJECT_ROOT" && git diff --submodule=short -- "$sub" 2>/dev/null)
    if [[ -n "$parent_diff" ]]; then
        error "HARD BLOCK: $sub pointer changed — reverting"
        (cd "$PROJECT_ROOT" && git checkout -- "$sub") 2>/dev/null
        return 1
    fi

    return 0
}

# Check a specific path inside a submodule (e.g. src/steps/ inside install-tests)
# Compares against BOTH working tree AND committed changes since baseline.
# Baseline is stored per-submodule before each iteration by snapshot_baselines().
hard_block_path() {
    local sub="$1"        # e.g. testing/install-tests
    local inner="$2"      # e.g. src/steps
    local sub_path="$PROJECT_ROOT/$sub"
    [[ -d "$sub_path/$inner" ]] || return 0

    # Check uncommitted changes
    local dirty
    dirty=$(cd "$sub_path" && git diff --name-only -- "$inner" 2>/dev/null)
    local untracked
    untracked=$(cd "$sub_path" && git ls-files --others --exclude-standard -- "$inner" 2>/dev/null)

    # Check committed changes since baseline
    local baseline_file="$RALPH_DIR/.baseline-$(echo "$sub" | tr '/' '-')"
    local committed=""
    if [[ -f "$baseline_file" ]]; then
        local baseline_sha
        baseline_sha=$(cat "$baseline_file")
        local current_sha
        current_sha=$(cd "$sub_path" && git rev-parse HEAD 2>/dev/null)
        if [[ "$baseline_sha" != "$current_sha" ]]; then
            committed=$(cd "$sub_path" && git diff --name-only "$baseline_sha" HEAD -- "$inner" 2>/dev/null)
        fi
    fi

    if [[ -n "$dirty" || -n "$untracked" || -n "$committed" ]]; then
        error "HARD BLOCK: $sub/$inner modified — reverting"
        # Revert uncommitted
        if [[ -n "$dirty" ]]; then
            (cd "$sub_path" && echo "$dirty" | xargs git checkout --) 2>/dev/null
        fi
        if [[ -n "$untracked" ]]; then
            (cd "$sub_path" && echo "$untracked" | xargs rm -f) 2>/dev/null
        fi
        # Revert committed changes: reset submodule to baseline
        if [[ -n "$committed" && -f "$baseline_file" ]]; then
            local baseline_sha
            baseline_sha=$(cat "$baseline_file")
            error "  Resetting $sub to baseline $baseline_sha"
            (cd "$sub_path" && git reset --hard "$baseline_sha") 2>/dev/null
            (cd "$PROJECT_ROOT" && git checkout -- "$sub") 2>/dev/null
        fi
        return 1
    fi

    return 0
}

# Snapshot submodule HEADs before an iteration starts.
# Called from run_iteration() so we can detect committed tampering.
snapshot_baselines() {
    for sub in testing/install-tests testing/cheat-guard distro-spec; do
        local sub_path="$PROJECT_ROOT/$sub"
        [[ -d "$sub_path/.git" || -f "$sub_path/.git" ]] || continue
        local sha
        sha=$(cd "$sub_path" && git rev-parse HEAD 2>/dev/null)
        echo "$sha" > "$RALPH_DIR/.baseline-$(echo "$sub" | tr '/' '-')"
    done
}

# Check a path inside a submodule, warn but don't revert
soft_warn_submodule() {
    local sub="$1"
    local sub_path="$PROJECT_ROOT/$sub"
    [[ -d "$sub_path/.git" || -f "$sub_path/.git" ]] || return 0

    local dirty
    dirty=$(cd "$sub_path" && git status --porcelain 2>/dev/null)
    local parent_diff
    parent_diff=$(cd "$PROJECT_ROOT" && git diff --submodule=short -- "$sub" 2>/dev/null)

    if [[ -n "$dirty" || -n "$parent_diff" ]]; then
        warn "NOTICE: $sub was modified this iteration (may be legitimate)"
        if [[ -n "$dirty" ]]; then
            echo "$dirty" | head -5 | while IFS= read -r line; do
                warn "  $line"
            done
        fi
        # Log to file for later review
        echo "[$(date '+%H:%M:%S')] $sub modified:" >> "$LOG_DIR/modifications.log"
        echo "$dirty" >> "$LOG_DIR/modifications.log" 2>/dev/null
        echo "$parent_diff" >> "$LOG_DIR/modifications.log" 2>/dev/null
        echo "---" >> "$LOG_DIR/modifications.log"
    fi
}

check_and_revert_protected() {
    local found_tampering=false

    # ── HARD BLOCKS: revert and fail ──

    # leviso — never touch
    if ! hard_block_submodule "leviso"; then
        found_tampering=true
    fi

    # cheat-guard — gutting protection is cheating
    if ! hard_block_submodule "testing/cheat-guard"; then
        found_tampering=true
    fi

    # distro-spec/src/levitate — never touch
    if ! hard_block_path "distro-spec" "src/levitate"; then
        found_tampering=true
    fi

    # install-tests step assertions — changing what "pass" means is cheating
    if ! hard_block_path "testing/install-tests" "src/steps"; then
        found_tampering=true
    fi

    # install-tests cheat macros usage (preflight.rs, executor.rs)
    if ! hard_block_path "testing/install-tests" "src/preflight.rs"; then
        found_tampering=true
    fi

    # ── SOFT WARNS: log for review, don't revert ──

    # install-tests other files (adding distro contexts, etc.)
    soft_warn_submodule "testing/install-tests"

    # rootfs-tests
    soft_warn_submodule "testing/rootfs-tests"

    # tools (might be fixing real bugs)
    for tool in tools/recstrap tools/recfstab tools/recchroot tools/recqemu tools/recuki tools/reciso; do
        soft_warn_submodule "$tool"
    done

    if [[ "$found_tampering" == "true" ]]; then
        ((REWARD_HACKS_BLOCKED++))
        error "━━━ REWARD HACK DETECTED AND REVERTED ━━━"
        error "Claude modified test assertions or protected code."
        error "Legitimate changes to other test files are preserved."
        return 1
    fi

    return 0
}

# =============================================================================
# Summary & cleanup
# =============================================================================

print_summary() {
    local total_elapsed=$((SECONDS - TOTAL_START_TIME))

    header "═══ Ralph Loop Summary ═══"
    echo -e "  Total time:          $(elapsed $total_elapsed)"
    echo -e "  Tasks completed:     $COMPLETED_TASKS"
    echo -e "  Failed iterations:   $FAILED_ITERATIONS"
    echo -e "  Timed out:           $TIMED_OUT_ITERATIONS"
    echo -e "  Rate limit waits:    $RATE_LIMITED_WAITS"
    echo -e "  Opus reviews:        $REVIEW_ITERATIONS"
    echo -e "  Reward hacks blocked: $REWARD_HACKS_BLOCKED"
    echo -e "  Logs:                $LOG_DIR/"
    echo ""
}

cleanup() {
    echo ""
    warn "Interrupted — cleaning up"
    print_summary
    restore_claude_md
    exit 130
}

trap cleanup INT TERM
trap restore_claude_md EXIT

# =============================================================================
# Build prompt with inline PRD/progress/learnings
# =============================================================================
# NOTE: We inline file contents instead of using @file syntax.
# @file may not resolve in -p mode, and inlining guarantees claude sees them.

build_prompt() {
    local phase="$1"
    local iteration="$2"
    local prd="$RALPH_DIR/${phase}-prd.md"
    local progress="$RALPH_DIR/${phase}-progress.txt"
    local learnings="$RALPH_DIR/${phase}-learnings.txt"

    local prd_content
    prd_content=$(cat "$prd")

    local progress_content
    progress_content=$(cat "$progress" 2>/dev/null)
    [[ -z "$progress_content" ]] && progress_content="(no previous progress)"

    local learnings_content
    learnings_content=$(cat "$learnings" 2>/dev/null)
    [[ -z "$learnings_content" ]] && learnings_content="(no learnings yet)"

    cat <<PROMPT
You are iteration $iteration of a Ralph loop building ${phase^}OS.
Work on ONE task only. Take the time you need, but don't wait on stuck commands.

═══════════════════════════════════════════
PRD — find the FIRST unchecked [ ] task
═══════════════════════════════════════════

$prd_content

═══════════════════════════════════════════
PROGRESS — what previous iterations did
═══════════════════════════════════════════

$progress_content

═══════════════════════════════════════════
LEARNINGS — patterns and gotchas
═══════════════════════════════════════════

$learnings_content

═══════════════════════════════════════════
INSTRUCTIONS
═══════════════════════════════════════════

1. Find the FIRST unchecked task in the PRD (marked with [ ]).
2. Implement ONLY that one task. Do not batch multiple tasks.
3. Run \`cargo check\` in the relevant crate to verify compilation.
4. If tests exist, run them.
5. Commit your changes in the relevant git submodule(s).
   - cd into the submodule directory first
   - git add the specific files you changed
   - Commit format: feat($phase): short description  OR  fix($phase): short description
6. Mark the completed task [x] in the PRD file at: $prd
7. Append a 1-2 line summary to: $progress
8. If you learned something non-obvious, append to: $learnings
9. If ALL tasks in the PRD are [x] and tests pass, output exactly on its own line:
   <promise>COMPLETE</promise>

CRITICAL RULES:
- ONE task per iteration. Stop after completing one task.
- ALWAYS commit. Small, frequent commits are the point.
- Do NOT modify leviso/ or distro-spec/src/levitate/.
- Do NOT modify test expectations to make them pass — fix the code.
- If a task is blocked, mark it as BLOCKED in the PRD, note why in progress, move to the next task.
- If you encounter a bug in shared code (distro-builder, distro-spec/shared), fix it in a SEPARATE commit.
- Do NOT modify CLAUDE.md — it is managed by the ralph loop.
PROMPT
}

# =============================================================================
# Build review prompt (opus — runs every REVIEW_EVERY iterations)
# =============================================================================

build_review_prompt() {
    local phase="$1"
    local iteration="$2"
    local prd="$RALPH_DIR/${phase}-prd.md"
    local progress="$RALPH_DIR/${phase}-progress.txt"
    local learnings="$RALPH_DIR/${phase}-learnings.txt"

    local prd_content
    prd_content=$(cat "$prd")

    local progress_content
    progress_content=$(cat "$progress" 2>/dev/null)
    [[ -z "$progress_content" ]] && progress_content="(no previous progress)"

    local learnings_content
    learnings_content=$(cat "$learnings" 2>/dev/null)
    [[ -z "$learnings_content" ]] && learnings_content="(no learnings yet)"

    # Get recent git log from writable submodules
    local recent_commits=""
    for sub in AcornOS IuppiterOS distro-spec distro-builder; do
        local sub_path="$PROJECT_ROOT/$sub"
        [[ -d "$sub_path" ]] || continue
        local log_output
        log_output=$(cd "$sub_path" && git log --oneline -"$REVIEW_EVERY" 2>/dev/null)
        if [[ -n "$log_output" ]]; then
            recent_commits+="
── $sub ──
$log_output
"
        fi
    done

    # Get recent diffs (summary only — keep prompt small)
    local recent_diffs=""
    for sub in AcornOS IuppiterOS distro-spec distro-builder; do
        local sub_path="$PROJECT_ROOT/$sub"
        [[ -d "$sub_path" ]] || continue
        local diff_stat
        diff_stat=$(cd "$sub_path" && git diff --stat "HEAD~${REVIEW_EVERY}" HEAD 2>/dev/null)
        if [[ -n "$diff_stat" ]]; then
            recent_diffs+="
── $sub ──
$diff_stat
"
        fi
    done

    cat <<PROMPT
You are an Opus REVIEW iteration for ${phase^}OS (after iteration $iteration).
Your job is to REVIEW and FIX the last $REVIEW_EVERY haiku iterations. Do NOT pick up new PRD tasks.

═══════════════════════════════════════════
RECENT COMMITS (last $REVIEW_EVERY iterations)
═══════════════════════════════════════════

$recent_commits

═══════════════════════════════════════════
RECENT CHANGES (diff summary)
═══════════════════════════════════════════

$recent_diffs

═══════════════════════════════════════════
PRD — current state (for context only)
═══════════════════════════════════════════

$prd_content

═══════════════════════════════════════════
PROGRESS
═══════════════════════════════════════════

$progress_content

═══════════════════════════════════════════
LEARNINGS
═══════════════════════════════════════════

$learnings_content

═══════════════════════════════════════════
YOUR INSTRUCTIONS (Opus Review)
═══════════════════════════════════════════

You are the senior reviewer. The last $REVIEW_EVERY iterations were done by haiku (fast but sloppy).
Your job is targeted quality improvement — be surgical, not exhaustive.

DO:
1. Run \`cargo check\` across the workspace. Fix any compilation errors haiku left behind.
2. Read the code haiku wrote in the last $REVIEW_EVERY commits. Look for:
   - Logic bugs (wrong conditions, off-by-one, missing error handling)
   - Layer boundary violations (AcornOS code in distro-builder, etc.)
   - Hardcoded values that should come from distro-spec
   - Dead code or unused imports haiku forgot to clean up
3. If you find bugs, fix them and commit: fix($phase): description
4. If haiku marked a task [x] but the implementation is incomplete/wrong, unmark it [ ] and note why in progress.
5. Run \`cargo test\` if tests exist for the changed crates. Fix failures.
6. Append a review summary to: $progress
7. If you learned something, append to: $learnings

DO NOT:
- Pick up new PRD tasks. That's haiku's job.
- Refactor or restructure working code. If it compiles and is correct, leave it.
- Add features, documentation, or tests beyond what's needed to fix bugs.
- Spend time on style or formatting (pre-commit hooks handle that).

CRITICAL RULES:
- Do NOT modify leviso/ or distro-spec/src/levitate/.
- Do NOT modify test expectations — fix the code.
- Do NOT modify CLAUDE.md — it is managed by the ralph loop.
- Keep your changes SMALL. You are expensive. Fix what's broken, nothing more.
PROMPT
}

# =============================================================================
# Run one review iteration (opus)
# =============================================================================

run_review() {
    local phase="$1"
    local after_iteration="$2"
    local logfile="$LOG_DIR/${phase}-review-$(printf '%03d' "$after_iteration").log"

    local iter_start=$SECONDS

    header "━━━ OPUS REVIEW after iteration $after_iteration [$phase] ━━━"
    dim "  Time: $(date '+%H:%M:%S')  |  Log: $logfile"
    dim "  Model: $REVIEW_MODEL  |  Budget: \$${REVIEW_BUDGET}"
    echo ""

    # Snapshot baselines (for anti-hack guard)
    snapshot_baselines

    local prompt
    prompt=$(build_review_prompt "$phase" "$after_iteration")

    # Run opus review
    local exit_code=0
    local rate_limit_retries=0

    while true; do
        exit_code=0
        timeout "$ITERATION_TIMEOUT" claude \
            --model "$REVIEW_MODEL" \
            --print \
            --dangerously-skip-permissions \
            --no-session-persistence \
            --max-budget-usd "$REVIEW_BUDGET" \
            --verbose \
            -p "$prompt" \
            2>&1 | tee "$logfile" || exit_code=$?

        if [[ $exit_code -ne 0 ]] && grep -qi 'rate.limit\|too many requests\|429\|overloaded' "$logfile" 2>/dev/null; then
            ((rate_limit_retries++))
            ((RATE_LIMITED_WAITS++))
            if [[ $rate_limit_retries -ge $MAX_RATE_LIMIT_RETRIES ]]; then
                warn "Review rate limited $rate_limit_retries times — skipping"
                break
            fi
            warn "Rate limited — waiting ${RATE_LIMIT_WAIT}s before retry"
            sleep "$RATE_LIMIT_WAIT"
            continue
        fi
        break
    done

    local iter_elapsed=$((SECONDS - iter_start))
    echo ""
    dim "──────────────────────────────────────────"

    if [[ $exit_code -eq 124 ]]; then
        warn "Review TIMED OUT after $(elapsed $iter_elapsed)"
        return
    fi

    if [[ $exit_code -ne 0 ]]; then
        warn "Review exited with code $exit_code after $(elapsed $iter_elapsed)"
        return
    fi

    # Anti-hack guard applies to reviews too
    verify_claude_md "$phase"
    if ! check_and_revert_protected; then
        warn "Review attempted reward hacking — changes reverted"
        return
    fi

    ((REVIEW_ITERATIONS++))
    log "Review completed in $(elapsed $iter_elapsed)"
}

# =============================================================================
# Run one iteration (haiku)
# =============================================================================

run_iteration() {
    local phase="$1"
    local iteration="$2"
    local max="$3"
    local logfile="$LOG_DIR/${phase}-$(printf '%03d' "$iteration").log"

    local iter_start=$SECONDS

    header "━━━ Iteration $iteration / $max [$phase] ━━━"
    dim "  Time: $(date '+%H:%M:%S')  |  Log: $logfile"
    dim "  Budget: \$${MAX_BUDGET_PER_ITERATION}/iter  |  Timeout: ${ITERATION_TIMEOUT}s"

    local prd="$RALPH_DIR/${phase}-prd.md"
    local done_count
    done_count=$(count_checked "$prd")
    local todo_count
    todo_count=$(count_unchecked "$prd")
    dim "  PRD: $done_count done, $todo_count remaining"
    echo ""

    # Snapshot submodule baselines before iteration (for committed tampering detection)
    snapshot_baselines

    # Snapshot progress hash before iteration
    local progress_before
    progress_before=$(md5_file "$RALPH_DIR/${phase}-progress.txt")

    # Build prompt
    local prompt
    prompt=$(build_prompt "$phase" "$iteration")

    # Run claude with timeout, stream to terminal and log
    # Retry on rate limits with backoff
    local exit_code=0
    local rate_limit_retries=0

    while true; do
        exit_code=0
        timeout "$ITERATION_TIMEOUT" claude \
            --model "$MODEL" \
            --print \
            --dangerously-skip-permissions \
            --no-session-persistence \
            --max-budget-usd "$MAX_BUDGET_PER_ITERATION" \
            --verbose \
            -p "$prompt" \
            2>&1 | tee "$logfile" || exit_code=$?

        # Check for rate limit in output
        if [[ $exit_code -ne 0 ]] && grep -qi 'rate.limit\|too many requests\|429\|overloaded' "$logfile" 2>/dev/null; then
            ((rate_limit_retries++))
            ((RATE_LIMITED_WAITS++))
            if [[ $rate_limit_retries -ge $MAX_RATE_LIMIT_RETRIES ]]; then
                error "Rate limited $rate_limit_retries times — giving up on iteration $iteration"
                break
            fi
            warn "Rate limited — waiting ${RATE_LIMIT_WAIT}s before retry ($rate_limit_retries/$MAX_RATE_LIMIT_RETRIES)"
            sleep "$RATE_LIMIT_WAIT"
            continue
        fi

        break  # not rate limited, proceed
    done

    local iter_elapsed=$((SECONDS - iter_start))

    echo ""
    dim "──────────────────────────────────────────"

    # Handle timeout (exit code 124 from timeout command)
    if [[ $exit_code -eq 124 ]]; then
        warn "Iteration $iteration TIMED OUT after $(elapsed $iter_elapsed)"
        ((TIMED_OUT_ITERATIONS++))
        echo "TIMED OUT after ${ITERATION_TIMEOUT}s" >> "$RALPH_DIR/${phase}-progress.txt"
        return 2  # timeout
    fi

    # Handle other errors
    if [[ $exit_code -ne 0 ]]; then
        warn "Iteration $iteration exited with code $exit_code after $(elapsed $iter_elapsed)"
        ((FAILED_ITERATIONS++))
        return 1  # error but continue
    fi

    log "Iteration $iteration completed in $(elapsed $iter_elapsed)"

    # Verify CLAUDE.md wasn't tampered with
    verify_claude_md "$phase"

    # Anti-reward-hack: revert any changes to protected submodules
    if ! check_and_revert_protected; then
        warn "Iteration $iteration attempted reward hacking — treating as failed"
        ((FAILED_ITERATIONS++))
        echo "REWARD HACK: modified protected test/tool code — reverted" >> "$RALPH_DIR/${phase}-progress.txt"
        return 1
    fi

    # Check progress changed
    local progress_after
    progress_after=$(md5_file "$RALPH_DIR/${phase}-progress.txt")
    if [[ "$progress_before" != "$progress_after" ]]; then
        local new_done
        new_done=$(count_checked "$prd")
        local delta=$((new_done - done_count))
        if [[ $delta -gt 0 ]]; then
            success "+$delta task(s) completed this iteration"
            COMPLETED_TASKS=$((COMPLETED_TASKS + delta))
        fi
    else
        warn "Progress file unchanged — possible stagnation"
    fi

    # Show latest progress
    echo ""
    dim "Latest progress:"
    tail -3 "$RALPH_DIR/${phase}-progress.txt" 2>/dev/null | while read -r line; do
        dim "  $line"
    done
    echo ""

    # Check for completion signal
    if grep -q '<promise>COMPLETE</promise>' "$logfile" 2>/dev/null; then
        return 0  # done!
    fi

    return 1  # not done, keep looping
}

# =============================================================================
# Run phase loop
# =============================================================================

run_phase() {
    local phase="$1"
    local max_iterations="${2:-$DEFAULT_MAX_ITERATIONS}"
    local phase_start=$SECONDS

    header "═══ Starting phase: $phase (max $max_iterations iterations) ═══"

    install_claude_md "$phase"
    install_hooks
    mkdir -p "$LOG_DIR"

    local stagnant_count=0
    local completed=false

    for ((i=1; i<=max_iterations; i++)); do
        local progress_before
        progress_before=$(md5_file "$RALPH_DIR/${phase}-progress.txt")

        local iter_result=0
        run_iteration "$phase" "$i" "$max_iterations" || iter_result=$?

        case $iter_result in
            0)
                # COMPLETE signal received
                success "Phase $phase COMPLETE after $i iterations!"
                completed=true
                break
                ;;
            2)
                # Timeout — don't count toward stagnation, just continue
                ;;
            *)
                # Check stagnation
                local progress_after
                progress_after=$(md5_file "$RALPH_DIR/${phase}-progress.txt")
                if [[ "$progress_before" == "$progress_after" ]]; then
                    ((stagnant_count++))
                    warn "Stagnant iteration $stagnant_count / $MAX_STAGNANT_ITERATIONS"
                    if [[ $stagnant_count -ge $MAX_STAGNANT_ITERATIONS ]]; then
                        error "Aborting: $MAX_STAGNANT_ITERATIONS consecutive iterations with no progress"
                        error "Review logs at: $LOG_DIR/"
                        error "Resume with: $0 $phase $((max_iterations - i))"
                        break
                    fi
                else
                    stagnant_count=0  # reset on progress
                fi
                ;;
        esac

        # Opus review every REVIEW_EVERY iterations
        if [[ $((i % REVIEW_EVERY)) -eq 0 && $i -lt $max_iterations ]]; then
            run_review "$phase" "$i"
            dim "Cooling down ${COOLDOWN_SECONDS}s..."
            sleep "$COOLDOWN_SECONDS"
        fi

        # Cooldown between iterations
        if [[ $i -lt $max_iterations ]]; then
            dim "Cooling down ${COOLDOWN_SECONDS}s..."
            sleep "$COOLDOWN_SECONDS"
        fi
    done

    local phase_elapsed=$((SECONDS - phase_start))
    echo ""
    log "Phase $phase finished in $(elapsed $phase_elapsed)"

    if [[ "$completed" != "true" ]]; then
        local prd="$RALPH_DIR/${phase}-prd.md"
        warn "Phase $phase incomplete: $(count_checked "$prd") done, $(count_unchecked "$prd") remaining"
    fi
}

# =============================================================================
# Main
# =============================================================================

main() {
    local command="${1:-}"
    local max_iterations="${2:-$DEFAULT_MAX_ITERATIONS}"

    if [[ -z "$command" ]]; then
        echo "Usage: $0 <acorn|iuppiter|all> [max_iterations]"
        echo ""
        echo "Commands:"
        echo "  acorn       Build AcornOS (desktop-ready Alpine variant)"
        echo "  iuppiter    Build IuppiterOS (headless refurbishment appliance)"
        echo "  all         AcornOS first, then IuppiterOS"
        echo ""
        echo "Options:"
        echo "  max_iterations  Max iterations per phase (default: $DEFAULT_MAX_ITERATIONS)"
        echo ""
        echo "Config (edit at top of script):"
        echo "  MODEL=$MODEL"
        echo "  ITERATION_TIMEOUT=${ITERATION_TIMEOUT}s"
        echo "  COOLDOWN_SECONDS=${COOLDOWN_SECONDS}s"
        echo "  MAX_BUDGET_PER_ITERATION=\$${MAX_BUDGET_PER_ITERATION}/iter"
        echo "  REVIEW_MODEL=$REVIEW_MODEL (every $REVIEW_EVERY iters, \$$REVIEW_BUDGET/review)"
        echo "  MAX_STAGNANT_ITERATIONS=$MAX_STAGNANT_ITERATIONS"
        exit 1
    fi

    preflight

    header "Ralph Loop"
    echo "  Phase:      $command"
    echo "  Model:      $MODEL"
    echo "  Iterations: $max_iterations per phase"
    echo "  Timeout:    ${ITERATION_TIMEOUT}s per iteration"
    echo "  Budget:     \$${MAX_BUDGET_PER_ITERATION} per iteration"
    echo "  Review:     $REVIEW_MODEL every $REVIEW_EVERY iterations (\$${REVIEW_BUDGET}/review)"
    echo "  Stagnation: abort after $MAX_STAGNANT_ITERATIONS stuck iterations"
    echo "  Logs:       $LOG_DIR/"
    echo "  CWD:        $PROJECT_ROOT"
    echo ""
    warn "Running with --dangerously-skip-permissions (unattended mode)"
    echo ""

    case "$command" in
        acorn)
            run_phase "acorn" "$max_iterations"
            ;;
        iuppiter)
            run_phase "iuppiter" "$max_iterations"
            ;;
        all)
            run_phase "acorn" "$max_iterations"
            echo ""
            log "AcornOS phase done. Switching to IuppiterOS..."
            echo ""
            run_phase "iuppiter" "$max_iterations"
            ;;
        *)
            error "Unknown command: $command"
            exit 1
            ;;
    esac

    print_summary
    restore_claude_md
}

main "$@"
