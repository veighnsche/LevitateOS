#!/bin/sh
# IuppiterOS test mode instrumentation
# Activates on serial console (ttyS0) - test harness environment
# IuppiterOS is headless, so serial console IS the default interface

# Only run in interactive shells
case "$-" in
    *i*) ;;
    *) return ;;
esac

# IuppiterOS is headless: serial console on ttyS0 is the ONLY interface
# Any interactive shell IS test mode
if [ "$(tty)" = "/dev/ttyS0" ]; then
    export IUPPITER_TEST_MODE=1
else
    # Should not happen on IuppiterOS (headless)
    return
fi

# ═══════════════════════════════════════════════════════════════════
# TEST MODE ACTIVE - Emit structured markers for install-tests harness
# ═══════════════════════════════════════════════════════════════════

# Disable command echo on serial console to prevent output contamination
stty -echo 2>/dev/null

# Command tracking (ash-compatible - no DEBUG trap, simpler approach)
_IUPPITER_CMD_ID=""

# Generate command ID
_iuppiter_cmd_id() {
    # Use /proc/uptime for millisecond-ish precision (ash doesn't have date +%3N)
    cut -d. -f1 /proc/uptime 2>/dev/null || date +%s
}

# Post-command hook via PS1 (ash doesn't have PROMPT_COMMAND)
# We emit markers in a function called from PS1
_iuppiter_prompt() {
    local exit_code=$?

    # Emit command end marker if we had a command
    if [ -n "$_IUPPITER_CMD_ID" ]; then
        echo "___CMD_END_${_IUPPITER_CMD_ID}_${exit_code}___"
        _IUPPITER_CMD_ID=""
    fi

    # Emit prompt marker - tells test harness shell is ready
    echo "___PROMPT___"
}

# Pre-command hook - called by typing commands
# Since ash doesn't have DEBUG trap, we use a wrapper approach
_iuppiter_run() {
    _IUPPITER_CMD_ID=$(_iuppiter_cmd_id)
    echo "___CMD_START_${_IUPPITER_CMD_ID}_$*___"
    "$@"
}

# Set PS1 to emit markers
PS1='$(_iuppiter_prompt)# '

# Signal shell is ready - test harness waits for this
echo "___SHELL_READY___"
# Emit initial prompt marker
echo "___PROMPT___"

# Provide alias for wrapped command execution (optional - for explicit marking)
alias run='_iuppiter_run'
