#!/bin/sh
# IuppiterOS live ISO documentation helper
# IuppiterOS is headless: serial console (ttyS0) is the only interface
# This script only exists in live-overlay, so if it's running, we're in live mode

# Only run in interactive shells
case "$-" in
    *i*) ;;
    *) return ;;
esac

# Only run on serial console (IuppiterOS is headless)
[ "$(tty)" != "/dev/ttyS0" ] && return

# Print quick reference on first login
cat << 'EOF'

 ___                  _ _              ___  ____
|_ _|_   _ _ __  _ __(_) |_ ___ _ __ / _ \/ ___|
 | || | | | '_ \| '_ \ | __/ _ \ '__| | | \___ \
 | || |_| | |_) | |_) | | ||  __/ |  | |_| |___) |
|___|\__,_| .__/| .__/|_|\__\___|_|   \___/|____/
          |_|   |_|

IuppiterOS Refurbishment Server - Live Mode

Quick Start:
  recstrap /mnt         Install IuppiterOS to /mnt
  recfstab /mnt         Generate fstab
  recchroot /mnt        Chroot into installation

Refurbishment Tools:
  smartctl -a /dev/sdX  SMART health check
  hdparm -I /dev/sdX    Drive identification
  lsscsi                List SCSI devices
  sg_inq /dev/sgN       SCSI inquiry

Network:
  ip addr               Show IP addresses

Documentation:
  https://levitateos.org/iuppiter/docs

EOF
