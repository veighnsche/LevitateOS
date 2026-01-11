# Master Plan: LevitateOS General-Purpose OS

**Created**: 2026-01-10
**Status**: Planning Complete
**Goal**: Run any Unix program without modification

---

## Vision

LevitateOS becomes a **general-purpose Unix-compatible operating system**. Users can download a Linux binary and run it. Programs compiled for Linux just work.

```bash
# The test: Can a user do this?
$ wget https://example.com/some-linux-binary
$ chmod +x some-linux-binary
$ ./some-linux-binary
# It just works.
```

---

## Epic Overview

| Epic | TEAM | Description | Status |
|------|------|-------------|--------|
| **Epic 1** | TEAM_400 | Process Model (fork/exec/wait) | Design Complete |
| **Epic 2** | TEAM_401 | Filesystem Hierarchy Standard | Design Complete |
| **Epic 3** | TEAM_402 | Disk-Based Root Filesystem | Design Complete |
| **Epic 4** | TEAM_405 | Users & Permissions | Design Complete |
| **Epic 5** | - | Signals & Job Control | Not Started |
| **Epic 6** | - | Networking | Not Started |

---

## Epic 1: Process Model (TEAM_400)

**Goal**: Programs can fork, exec, and wait like on Linux.

### Scope

| Component | Description |
|-----------|-------------|
| fork() | Clone process with memory copy |
| execve() | Replace process image with new program |
| wait4()/waitpid() | Wait for child process |
| exit_group() | Terminate process |
| getpid()/getppid() | Process identity |
| Orphan handling | Reparent to PID 1 |

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| fork() memory | Eager copy | CoW adds complexity; optimize when profiling shows need |
| Executable lookup | VFS resolution | Standard behavior |
| Orphan reparenting | Yes, to PID 1 | Standard Unix semantics |
| poll/ppoll | Both | Maximum compatibility |

### Dependencies

- None (foundational)

### Deliverables

- [ ] fork() syscall clones process
- [ ] execve() loads and runs ELF binaries
- [ ] wait4() blocks until child exits
- [ ] Process tree maintained correctly
- [ ] Orphans reparented to init

---

## Epic 2: Filesystem Hierarchy Standard (TEAM_401)

**Goal**: Standard Unix paths work correctly.

### Scope

| Component | Description |
|-----------|-------------|
| Merged /usr | /bin → /usr/bin, /sbin → /usr/sbin, /lib → /usr/lib |
| devtmpfs | Dynamic device filesystem at /dev |
| Extended /dev | null, zero, urandom, tty, console, stdin, stdout, stderr, fd/ |
| /proc/self | Minimal procfs for self-introspection |
| /etc files | passwd, group, shadow, shells, profile, hostname |
| Mount sequence | devtmpfs, tmpfs at /tmp, /run, /var/log |

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| /usr layout | Merged | Modern standard; every major distro has moved here |
| /dev implementation | devtmpfs | PTY needs dynamic device creation |
| /dev nodes | Extended (13+) | /dev/stdin, stdout, stderr widely used |
| procfs | Minimal /proc/self | /proc/self/exe needed for std::env::current_exe() |
| Random quality | RDRAND on x86_64 | One instruction, vastly better than PRNG |

### Dependencies

- TEAM_400 (fork/exec for init)

### Deliverables

- [ ] /bin, /sbin, /lib are symlinks to /usr/*
- [x] /dev/null, /dev/zero, /dev/urandom work (TEAM_431: devtmpfs implemented)
- [ ] /dev/stdin, /dev/stdout, /dev/stderr work
- [ ] /proc/self/exe returns executable path
- [ ] /etc/passwd, /etc/group parsed correctly
- [x] Mount sequence executes at boot (devtmpfs mounts at /dev)

---

## Epic 3: Disk-Based Root Filesystem (TEAM_402)

**Goal**: Install LevitateOS to disk with persistence.

### Scope

| Component | Description |
|-----------|-------------|
| pivot_root | Linux-compatible syscall (155/41) |
| MBR parsing | Detect disk partitions |
| ext4 write | Full read-write ext4 support |
| Partition nodes | /dev/vda1, etc. |
| Root detection | Auto-detect installed OS |
| Installer | Basic installation utility |

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| pivot_root | Linux-compatible | ABI compatibility; scripts expect it |
| Root filesystem | ext4 | FAT32 has no symlinks/permissions—unacceptable |
| Partition table | MBR | 1GB images don't need GPT |
| Bootloader | ISO only | Scope separation; disk boot is TEAM_403 |
| Boot mode | Auto-detect | Best UX; detect /sbin/init on disk |

### Critical Path

**ext4 write support is a prerequisite**. This is blocking work.

### Dependencies

- TEAM_400 (fork/exec for installer)
- TEAM_401 (FHS structure on disk)

### Deliverables

- [ ] pivot_root() syscall works
- [ ] MBR partitions detected
- [ ] /dev/vda1 device node created
- [ ] ext4 write support (create files, directories)
- [ ] Init detects installed OS and switches root
- [ ] Installer creates working installation
- [ ] Changes persist across reboot

---

## Epic 4: Users & Permissions (TEAM_405)

**Goal**: Real Unix multi-user security.

### Scope

| Component | Description |
|-----------|-------------|
| Process credentials | ruid/euid/suid, rgid/egid/sgid, groups |
| File permissions | Mode bits (rwxrwxrwx), owner, group |
| VFS enforcement | Permission checks on all operations |
| Identity syscalls | getuid, setuid, getgroups, setgroups, etc. |
| File syscalls | chmod, chown, umask, access |
| setuid/setgid | Execute as file owner |
| Authentication | /etc/shadow with SHA-512 crypt |
| su utility | Switch user identity |

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Password auth | Shadow file | Real auth from start |
| Boot identity | Root (UID 0) | Init needs full privileges |
| Capabilities | Traditional UID 0 | 40+ capability bits are complex |
| Default user | "user" (UID 1000) | Standard first-user UID |
| Groups | Full supplementary | Programs expect getgroups() |
| Privilege escalation | su only | sudo's config is complex |

### Dependencies

- TEAM_400 (fork/exec for su)
- TEAM_401 (/etc files)
- TEAM_402 (ext4 stores permissions)

### Deliverables

- [ ] Processes have UID/GID credentials
- [ ] Files have owner/group/mode
- [ ] Permission checks enforced
- [ ] chmod/chown work
- [ ] Root bypasses all checks
- [ ] setuid binaries work
- [ ] su switches user

---

## Epic 5: Signals & Job Control (Future)

**Goal**: Ctrl+C, background jobs, proper signal handling.

### Scope (Preliminary)

| Component | Description |
|-----------|-------------|
| Signal delivery | SIGINT, SIGTERM, SIGKILL, SIGCHLD, etc. |
| Signal handlers | sigaction(), signal() |
| Process groups | setpgid(), getpgid() |
| Sessions | setsid(), getsid() |
| Controlling terminal | TIOCSCTTY, TIOCGPGRP |
| Job control | fg, bg, jobs in shell |

### Dependencies

- TEAM_400 (process model)
- TEAM_401 (/dev/tty)

---

## Epic 6: Networking (Future)

**Goal**: TCP/IP networking, sockets, DNS.

### Scope (Preliminary)

| Component | Description |
|-----------|-------------|
| Socket syscalls | socket, bind, listen, accept, connect |
| TCP/IP stack | Or use smoltcp |
| VirtIO-net driver | Already exists |
| DNS resolution | /etc/resolv.conf, stub resolver |
| Network utilities | ping, wget, curl |

### Dependencies

- All previous epics

---

## Dependency Graph

```
                    ┌─────────────────┐
                    │   TEAM_400      │
                    │  Process Model  │
                    │  (fork/exec)    │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
     │  TEAM_401   │  │  TEAM_405   │  │  Signals    │
     │    FHS      │  │   Users     │  │  (Future)   │
     │  devtmpfs   │  │   Perms     │  │             │
     └──────┬──────┘  └──────┬──────┘  └─────────────┘
            │                │
            │                │
            ▼                │
     ┌─────────────┐         │
     │  TEAM_402   │◄────────┘
     │  Disk Root  │
     │  (ext4 rw)  │
     └──────┬──────┘
            │
            ▼
     ┌─────────────┐
     │  TEAM_403   │
     │    Disk     │
     │  Bootloader │
     │  (Future)   │
     └─────────────┘
```

---

## Implementation Order

### Phase 1: Foundation (TEAM_400)

1. Implement fork() with eager memory copy
2. Implement execve() with ELF loading
3. Implement wait4()/waitpid()
4. Implement exit_group()
5. Handle orphan reparenting to PID 1
6. Test: shell can run commands

### Phase 2: Filesystem Structure (TEAM_401)

1. Restructure initramfs with merged /usr
2. ~~Implement devtmpfs filesystem~~ ✅ TEAM_431
3. ~~Create standard device nodes~~ ✅ TEAM_431 (null, zero, full, urandom)
4. Implement /dev/fd/, stdin, stdout, stderr
5. Implement minimal /proc/self
6. Add /etc configuration files
7. ~~Update boot mount sequence~~ ✅ TEAM_431 (devtmpfs mounts at /dev)
8. Test: /bin/sh works, /dev/null works ✅ (cgull-test passes 19/19)

### Phase 3: Permissions (TEAM_405)

1. Add credentials to task struct
2. Add uid/gid/mode to all inodes
3. Implement permission checks in VFS
4. Implement getuid/geteuid/getgid/getegid
5. Implement chmod/chown/umask
6. Handle setuid bit in exec
7. Implement setuid/setgid syscalls
8. Parse /etc/passwd, /etc/shadow, /etc/group
9. Implement su utility
10. Test: permissions enforced, su works

### Phase 4: Persistence (TEAM_402)

1. Implement ext4 write support
2. Implement MBR partition parsing
3. Create partition device nodes
4. Implement pivot_root syscall
5. Update init with root detection
6. Implement installer utility
7. Resize disk image to 1GB
8. Test: install, reboot, changes persist

### Phase 5: Polish & Signals (Future)

1. Signal delivery infrastructure
2. Signal handlers (sigaction)
3. Process groups and sessions
4. Job control
5. Test: Ctrl+C works, jobs work

---

## Success Criteria

### Minimum Viable General-Purpose OS

- [ ] fork/exec/wait work correctly
- [ ] Standard paths exist (/bin/sh, /usr/bin/env)
- [ ] Device files work (/dev/null, /dev/urandom)
- [ ] File permissions enforced
- [ ] Multiple users supported
- [ ] Root can do anything, users restricted
- [ ] Install to disk works
- [ ] Changes persist across reboot

### Full General-Purpose OS

- [ ] All of the above, plus:
- [ ] Signals work (Ctrl+C)
- [ ] Job control works (bg, fg)
- [ ] Networking works (TCP/IP)
- [ ] Can run unmodified Linux binaries
- [ ] Can compile programs on the OS itself

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| ext4 write complexity | High | Critical | Start early, it's blocking |
| fork() memory bugs | Medium | High | Thorough testing |
| Permission bypass | Medium | Critical | Security audit |
| ABI incompatibility | Medium | High | Test with real programs |
| Scope creep | High | Medium | Stick to defined scope |

---

## Resource Allocation

| Epic | Estimated Effort | Priority |
|------|------------------|----------|
| TEAM_400 (Process) | 2-3 weeks | P0 |
| TEAM_401 (FHS) | 1-2 weeks | P0 |
| TEAM_405 (Permissions) | 2 weeks | P1 |
| TEAM_402 (Disk Root) | 3-4 weeks | P1 |
| Signals | 2 weeks | P2 |
| Networking | 4+ weeks | P3 |

---

## Documentation Index

### TEAM Files

| TEAM | File |
|------|------|
| TEAM_400 | `.teams/TEAM_400_feature_general_purpose_os.md` |
| TEAM_401 | `.teams/TEAM_401_feature_filesystem_hierarchy.md` |
| TEAM_402 | `.teams/TEAM_402_feature_disk_root_filesystem.md` |
| TEAM_405 | `.teams/TEAM_405_feature_users_permissions.md` |

### Planning Documents

| Epic | Directory |
|------|-----------|
| TEAM_400 | `docs/planning/general-purpose-os/` |
| TEAM_401 | `docs/planning/filesystem-hierarchy/` |
| TEAM_402 | `docs/planning/disk-root-filesystem/` |
| TEAM_405 | `docs/planning/users-permissions/` |

### Question Files

| TEAM | File |
|------|------|
| TEAM_400 | `docs/questions/TEAM_400_general_purpose_os.md` |
| TEAM_401 | `docs/questions/TEAM_401_filesystem_hierarchy.md` |
| TEAM_402 | `docs/questions/TEAM_402_disk_root_filesystem.md` |
| TEAM_405 | `docs/questions/TEAM_405_users_permissions.md` |

---

## Summary

This plan transforms LevitateOS from a demonstration kernel into a **real general-purpose operating system**. The key milestones are:

1. **Process Model** - fork/exec/wait (programs can spawn programs)
2. **FHS Compliance** - standard paths work (/bin/sh, /dev/null)
3. **Permissions** - multi-user security (chmod/chown, setuid)
4. **Persistence** - install to disk (ext4 write, pivot_root)

After these four epics, LevitateOS can:
- Run unmodified Unix programs
- Support multiple users with proper isolation
- Persist data across reboots
- Compete as a real operating system

**The hard part is ext4 write support.** Everything else builds on existing infrastructure.
