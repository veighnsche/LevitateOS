#!/bin/sh
# Auto-launch tmux with docs on live ISO boot
# Only runs on tty1, only if not already in tmux
# This script only exists in live-overlay, so if it's running, we're in live mode

# Skip if already in tmux
[ -n "$TMUX" ] && return

# Skip if not on tty1 (allow SSH and other ttys to be normal shells)
[ "$(tty)" != "/dev/tty1" ] && return

# Skip if tmux not available
command -v tmux >/dev/null 2>&1 || return

# Check if acorn-docs exists, otherwise just show welcome
if command -v acorn-docs >/dev/null 2>&1; then
    # Launch tmux with shell on left, docs on right
    exec tmux new-session -d -s live \; \
        set-option -g prefix None \; \
        set-option -g mouse on \; \
        set-option -g status-style 'bg=black,fg=white' \; \
        set-option -g status-left '' \; \
        set-option -g status-right ' Shift+Tab: switch | Ctrl+Left/Right: resize ' \; \
        set-option -g status-right-length 60 \; \
        bind-key -n BTab select-pane -t :.+ \; \
        bind-key -n C-Left resize-pane -L 5 \; \
        bind-key -n C-Right resize-pane -R 5 \; \
        split-window -h 'acorn-docs' \; \
        select-pane -t 0 \; \
        attach-session -t live
else
    # No docs available - just print welcome message
    cat << 'EOF'

    _                          ___  ____
   / \   ___ ___  _ __ _ __   / _ \/ ___|
  / _ \ / __/ _ \| '__| '_ \ | | | \___ \
 / ___ \ (_| (_) | |  | | | || |_| |___) |
/_/   \_\___\___/|_|  |_| |_| \___/|____/

Welcome to AcornOS Live!

Quick Start:
  recstrap /mnt         Install AcornOS to /mnt
  recfstab /mnt         Generate fstab
  recchroot /mnt        Chroot into installation

Network:
  ip addr              Show IP addresses
  iwctl                WiFi configuration (iwd)

Documentation:
  https://levitateos.org/acorn/docs

EOF
fi
