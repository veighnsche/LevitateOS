# POSIX Terminal (TTY) Specification

**Source**: POSIX.1-2008, Linux termios(3) man page, IEEE Std 1003.1

This document defines what a **real terminal** (not a terminal emulator) must implement.

---

## 1. Core Data Structure: `struct termios`

```c
struct termios {
    tcflag_t c_iflag;    // Input modes
    tcflag_t c_oflag;    // Output modes  
    tcflag_t c_cflag;    // Control modes (hardware)
    tcflag_t c_lflag;    // Local modes (line discipline)
    cc_t c_cc[NCCS];     // Special characters
};
```

---

## 2. Input Mode Flags (`c_iflag`)

| Flag | Description |
|------|-------------|
| `IGNBRK` | Ignore BREAK condition |
| `BRKINT` | BREAK causes SIGINT to foreground process group |
| `IGNPAR` | Ignore framing/parity errors |
| `PARMRK` | Mark parity errors with \377 \0 prefix |
| `INPCK` | Enable input parity checking |
| `ISTRIP` | Strip 8th bit (make 7-bit) |
| `INLCR` | Translate NL to CR on input |
| `IGNCR` | Ignore CR on input |
| `ICRNL` | **Translate CR to NL on input** (common default) |
| `IUCLC` | Map uppercase to lowercase |
| `IXON` | Enable XON/XOFF flow control on output |
| `IXANY` | Any char restarts stopped output |
| `IXOFF` | Enable XON/XOFF flow control on input |
| `IMAXBEL` | Ring bell when input queue full |
| `IUTF8` | Input is UTF-8 (for proper char-erase) |

---

## 3. Output Mode Flags (`c_oflag`)

| Flag | Description |
|------|-------------|
| `OPOST` | Enable output processing |
| `OLCUC` | Map lowercase to uppercase on output |
| `ONLCR` | **Map NL to CR-NL on output** (common default) |
| `OCRNL` | Map CR to NL on output |
| `ONOCR` | Don't output CR at column 0 |
| `ONLRET` | NL does carriage-return |
| `OFILL` | Send fill chars for delay |
| `NLDLY` | Newline delay mask (NL0, NL1) |
| `CRDLY` | Carriage return delay (CR0-CR3) |
| `TABDLY` | Tab delay (TAB0-TAB3, XTABS=expand to spaces) |
| `BSDLY` | Backspace delay |
| `VTDLY` | Vertical tab delay |
| `FFDLY` | Form feed delay |

---

## 4. Control Mode Flags (`c_cflag`) - Hardware

| Flag | Description |
|------|-------------|
| `CSIZE` | Character size mask (CS5, CS6, CS7, CS8) |
| `CSTOPB` | 2 stop bits (else 1) |
| `CREAD` | Enable receiver |
| `PARENB` | Enable parity |
| `PARODD` | Odd parity (else even) |
| `HUPCL` | Hang up on last close |
| `CLOCAL` | Ignore modem control lines |
| `CRTSCTS` | Hardware flow control (RTS/CTS) |

---

## 5. Local Mode Flags (`c_lflag`) - **LINE DISCIPLINE**

| Flag | Description |
|------|-------------|
| `ISIG` | **Enable signal generation** (INTR→SIGINT, QUIT→SIGQUIT, SUSP→SIGTSTP) |
| `ICANON` | **Enable canonical (line) mode** |
| `ECHO` | **Echo input characters** |
| `ECHOE` | ERASE erases char visually (backspace-space-backspace) |
| `ECHOK` | KILL erases line |
| `ECHONL` | Echo NL even if ECHO off |
| `ECHOCTL` | Echo control chars as ^X |
| `ECHOPRT` | Print chars as erased |
| `ECHOKE` | KILL erases each char |
| `NOFLSH` | Don't flush queues on signal |
| `TOSTOP` | **SIGTTOU for background writes** |
| `IEXTEN` | Enable extended input processing |

---

## 6. Special Characters (`c_cc[]`)

### Signal Characters (when `ISIG` set)

| Index | Default | Name | Action |
|-------|---------|------|--------|
| `VINTR` | Ctrl-C (0x03) | Interrupt | Send **SIGINT** to foreground pgrp |
| `VQUIT` | Ctrl-\ (0x1C) | Quit | Send **SIGQUIT** (core dump) |
| `VSUSP` | Ctrl-Z (0x1A) | Suspend | Send **SIGTSTP** (stop process) |
| `VDSUSP` | Ctrl-Y | Delayed Suspend | SIGTSTP when read |

### Line Editing Characters (when `ICANON` set)

| Index | Default | Name | Action |
|-------|---------|------|--------|
| `VERASE` | DEL (0x7F) or Ctrl-H | Erase | Delete previous character |
| `VWERASE` | Ctrl-W | Word Erase | Delete previous word |
| `VKILL` | Ctrl-U | Kill | Delete entire line |
| `VREPRINT` | Ctrl-R | Reprint | Reprint unread chars |
| `VLNEXT` | Ctrl-V | Literal Next | Quote next char |

### Line Delimiters (when `ICANON` set)

| Index | Default | Name | Action |
|-------|---------|------|--------|
| `VEOF` | Ctrl-D (0x04) | End of File | Flush buffer, EOF if empty |
| `VEOL` | NUL | End of Line | Additional line delimiter |
| `VEOL2` | NUL | End of Line 2 | Yet another delimiter |

### Flow Control (when `IXON`/`IXOFF` set)

| Index | Default | Name | Action |
|-------|---------|------|--------|
| `VSTOP` | Ctrl-S (0x13) | Stop | Stop output |
| `VSTART` | Ctrl-Q (0x11) | Start | Resume output |

### Non-Canonical Mode

| Index | Name | Description |
|-------|------|-------------|
| `VMIN` | MIN | Minimum chars for read |
| `VTIME` | TIME | Timeout in 1/10 seconds |

---

## 7. Two Operating Modes

### Canonical Mode (`ICANON` set) - DEFAULT
- Input available **line by line**
- Line delimiters: NL, EOF, EOL, EOL2
- **Line editing enabled**: ERASE, KILL, WERASE, REPRINT, LNEXT
- `read()` returns at most one line
- Max line length: 4096 chars

### Non-Canonical Mode (`ICANON` unset) - RAW-ish
- Input available **immediately** (char by char)
- No line editing
- `MIN` and `TIME` control read behavior:

| MIN | TIME | Behavior |
|-----|------|----------|
| 0 | 0 | Polling: return immediately |
| >0 | 0 | Blocking: wait for MIN chars |
| 0 | >0 | Timeout: return after TIME or 1 char |
| >0 | >0 | Interbyte timeout |

### Raw Mode (`cfmakeraw()`)
Disables ALL processing:
```c
c_iflag &= ~(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
c_oflag &= ~OPOST;
c_lflag &= ~(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
c_cflag &= ~(CSIZE | PARENB);
c_cflag |= CS8;
```

---

## 8. Job Control & Process Groups

### Controlling Terminal
- Each process has 0 or 1 controlling terminal
- Inherited from parent
- Acquired by `open()` without `O_NOCTTY`

### Foreground Process Group
- Terminal has one **foreground process group**
- Signals from terminal go to foreground pgrp
- Set with `tcsetpgrp(fd, pgrp)`
- Get with `tcgetpgrp(fd)`

### Background Process Behavior
- **Background read**: generates `SIGTTIN` (stops process)
- **Background write**: generates `SIGTTOU` if `TOSTOP` set

---

## 9. Signals Generated by Terminal

| Condition | Signal | Default Action |
|-----------|--------|----------------|
| INTR char (Ctrl-C) | SIGINT | Terminate |
| QUIT char (Ctrl-\) | SIGQUIT | Core dump |
| SUSP char (Ctrl-Z) | SIGTSTP | Stop (can resume) |
| Background read | SIGTTIN | Stop |
| Background write | SIGTTOU | Stop |
| Hangup | SIGHUP | Terminate |

---

## 10. API Functions

### Terminal Attributes
```c
int tcgetattr(int fd, struct termios *t);           // Get attributes
int tcsetattr(int fd, int when, struct termios *t); // Set attributes
    // when: TCSANOW, TCSADRAIN, TCSAFLUSH
```

### Line Control
```c
int tcsendbreak(int fd, int duration);  // Send break
int tcdrain(int fd);                    // Wait for output
int tcflush(int fd, int queue);         // Flush queues
    // queue: TCIFLUSH, TCOFLUSH, TCIOFLUSH
int tcflow(int fd, int action);         // Suspend/resume
    // action: TCOOFF, TCOON, TCIOFF, TCION
```

### Process Group
```c
pid_t tcgetpgrp(int fd);                // Get foreground pgrp
int tcsetpgrp(int fd, pid_t pgrp);      // Set foreground pgrp
```

### Baud Rate
```c
speed_t cfgetispeed(struct termios *t);
speed_t cfgetospeed(struct termios *t);
int cfsetispeed(struct termios *t, speed_t speed);
int cfsetospeed(struct termios *t, speed_t speed);
```

---

## 11. What LevitateOS Currently Has vs Needs

### ✅ Have
- Basic stdin read
- Echo (manual in shell)
- SIGINT signal
- Foreground process tracking
- Ctrl+C detection

### ❌ Need for Full POSIX TTY
- [ ] `termios` structure and syscalls
- [ ] Canonical vs non-canonical modes
- [ ] Line editing (ERASE, KILL, WERASE)
- [ ] All special characters
- [ ] SIGQUIT (Ctrl-\), SIGTSTP (Ctrl-Z)
- [ ] Job control (SIGTTIN, SIGTTOU)
- [ ] Process groups
- [ ] Output processing (ONLCR, etc.)
- [ ] Flow control (Ctrl-S/Ctrl-Q)

---

## References

1. **POSIX.1-2008** (IEEE Std 1003.1-2008) - Chapter 11: General Terminal Interface
2. **Linux termios(3)** - https://man7.org/linux/man-pages/man3/termios.3.html
3. **Linux tty(4)** - https://man7.org/linux/man-pages/man4/tty.4.html
4. **The TTY demystified** - https://www.linusakesson.net/programming/tty/
