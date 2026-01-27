# TEAM_127: Visual Install Testing via Puppeteer + noVNC

## Status: COMPLETE

## Quick Reference (READ THIS FIRST)

### Step 1: Start VM and Bridge
```bash
# Terminal 1: QEMU with VNC
qemu-system-x86_64 -enable-kvm -m 4G -cpu host \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/edk2/ovmf/OVMF_CODE.fd \
  -cdrom output/levitateos.iso -vnc :0 -device virtio-vga -boot d -display none

# Terminal 2: websockify (noVNC must be cloned to /tmp/novnc first)
websockify 6080 localhost:5900 --web /tmp/novnc
```

### Step 2: Connect Puppeteer
```
puppeteer_navigate url="http://localhost:6080/vnc.html?autoconnect=true"
```
Wait 3 seconds for connection.

### Step 3: Send Commands
```
puppeteer_fill selector="#noVNC_keyboardinput" value="your command here\n"
```
- Always include `\n` at the end to press Enter
- That's it. This is the reliable method.

### Step 4: Take Screenshot
```
puppeteer_screenshot name="descriptive-name" width=1024 height=768
```

### One-Time Setup (if noVNC not present)
```bash
git clone --depth 1 https://github.com/novnc/noVNC.git /tmp/novnc
```

---

## Why This Works

- `#noVNC_keyboardinput` is noVNC's hidden textarea for mobile keyboard input
- `puppeteer_fill` types into it character by character, triggering real VNC keystrokes
- Don't use `puppeteer_evaluate` with keyboard events - it's unreliable

## Architecture
```
Puppeteer MCP ──► noVNC (browser) ──► websockify ──► QEMU VNC
```

## Verified Working (2026-01-27)
- `echo hello world` → Output: `hello world`
- `uname -a` → Shows kernel version
- Screenshots capture VM display correctly
