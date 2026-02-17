#!/bin/sh
# AcornOS test mode instrumentation
# Activates ONLY on serial console (ttyS0) - test harness environment
# Users on tty1 see normal behavior (+ docs from live-docs.sh)

# Only run in interactive shells
case "$-" in
    *i*) ;;
    *) return ;;
esac

# Detect test mode: serial console = test mode
if [ "$(tty)" = "/dev/ttyS0" ]; then
    export ACORN_TEST_MODE=1
else
    # Not test mode - exit early, let live-docs.sh handle normal UX
    return
fi

# ═══════════════════════════════════════════════════════════════════
# TEST MODE ACTIVE - Emit structured markers for install-tests harness
# ═══════════════════════════════════════════════════════════════════

# Disable command echo on serial console to prevent output contamination
stty -echo 2>/dev/null

# Command tracking (ash-compatible - no DEBUG trap, simpler approach)
_ACORN_CMD_ID=""

# Generate command ID
_acorn_cmd_id() {
    # Use /proc/uptime for millisecond-ish precision (ash doesn't have date +%3N)
    cut -d. -f1 /proc/uptime 2>/dev/null || date +%s
}

# Post-command hook via PS1 (ash doesn't have PROMPT_COMMAND)
# We emit markers in a function called from PS1
_acorn_prompt() {
    local exit_code=$?

    # Emit command end marker if we had a command
    if [ -n "$_ACORN_CMD_ID" ]; then
        echo "___CMD_END_${_ACORN_CMD_ID}_${exit_code}___"
        _ACORN_CMD_ID=""
    fi

    # Emit prompt marker - tells test harness shell is ready
    echo "___PROMPT___"
}

# Pre-command hook - called by typing commands
# Since ash doesn't have DEBUG trap, we use a wrapper approach
_acorn_run() {
    _ACORN_CMD_ID=$(_acorn_cmd_id)
    echo "___CMD_START_${_ACORN_CMD_ID}_$*___"
    "$@"
}

# Set PS1 to emit markers
PS1='$(_acorn_prompt)# '

# Signal shell is ready - test harness waits for this
echo "___SHELL_READY___"
# Emit initial prompt marker
echo "___PROMPT___"

# Provide alias for wrapped command execution (optional - for explicit marking)
alias run='_acorn_run'
