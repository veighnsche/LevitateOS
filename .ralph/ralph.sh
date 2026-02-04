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
DEFAULT_MAX_ITERATIONS=50
ITERATION_TIMEOUT=3600         # 60 minutes max per iteration (install-tests need QEMU)
COOLDOWN_SECONDS=5             # pause between iterations (rate limit protection)
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
    grep -c '^\- \[x\]' "$1" 2>/dev/null || echo 0
}

count_unchecked() {
    grep -c '^\- \[ \]' "$1" 2>/dev/null || echo 0
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
# Summary & cleanup
# =============================================================================

print_summary() {
    local total_elapsed=$((SECONDS - TOTAL_START_TIME))

    header "═══ Ralph Loop Summary ═══"
    echo -e "  Total time:          $(elapsed $total_elapsed)"
    echo -e "  Tasks completed:     $COMPLETED_TASKS"
    echo -e "  Failed iterations:   $FAILED_ITERATIONS"
    echo -e "  Timed out:           $TIMED_OUT_ITERATIONS"
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
You have a maximum of 15 minutes. Work on ONE task only.

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
# Run one iteration
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

    # Snapshot progress hash before iteration
    local progress_before
    progress_before=$(md5_file "$RALPH_DIR/${phase}-progress.txt")

    # Build prompt
    local prompt
    prompt=$(build_prompt "$phase" "$iteration")

    # Run claude with timeout, stream to terminal and log
    local exit_code=0
    timeout "$ITERATION_TIMEOUT" claude \
        --model "$MODEL" \
        --print \
        --dangerously-skip-permissions \
        --no-session-persistence \
        --max-budget-usd "$MAX_BUDGET_PER_ITERATION" \
        --verbose \
        -p "$prompt" \
        2>&1 | tee "$logfile" || exit_code=$?

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
