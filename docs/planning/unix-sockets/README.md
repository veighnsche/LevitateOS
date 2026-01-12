# Unix Domain Sockets Implementation Plan

## Goal

Implement Unix domain sockets (AF_UNIX) to enable Wayland compositor support and standard Unix IPC.

## Why This Matters

Wayland protocol requires Unix domain sockets for:
1. Client-compositor communication via `/run/wayland-0` socket
2. File descriptor passing (SCM_RIGHTS) for shared memory buffers
3. Credential passing (SCM_CREDENTIALS) for security

Many other Unix programs also depend on Unix sockets: D-Bus, systemd, X11, databases, etc.

## Current State

| Feature | Status |
|---------|--------|
| `socketpair()` | Stub - returns pipe pair |
| `socket()` | Not implemented |
| `bind()` | Not implemented |
| `listen()` | Not implemented |
| `accept()` | Not implemented |
| `connect()` | Not implemented |
| `sendmsg()`/`recvmsg()` | Not implemented |
| Socket file type in VFS | Not implemented |

## Phases

| Phase | Description | Estimated UoW |
|-------|-------------|---------------|
| 1 | Socket infrastructure & socketpair | 8 |
| 2 | Filesystem sockets (bind/listen/accept/connect) | 10 |
| 3 | Advanced features (fd passing, credentials) | 6 |

**Total: ~24 units of work**

## Success Criteria

1. `socketpair(AF_UNIX, SOCK_STREAM, 0, sv)` creates connected socket pair
2. Server can `bind()` to `/tmp/test.sock`, `listen()`, `accept()`
3. Client can `connect()` to socket path
4. `sendmsg()`/`recvmsg()` with SCM_RIGHTS passes file descriptors
5. Wayland test client can connect to compositor

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    User Space                           │
│  ┌─────────┐    ┌─────────┐    ┌─────────────────────┐ │
│  │ Wayland │    │ Client  │    │ Other Unix Programs │ │
│  │Compositor│    │  App    │    │  (dbus, etc.)       │ │
│  └────┬────┘    └────┬────┘    └──────────┬──────────┘ │
│       │              │                     │            │
│       └──────────────┼─────────────────────┘            │
│                      │ socket syscalls                  │
└──────────────────────┼──────────────────────────────────┘
                       │
┌──────────────────────┼──────────────────────────────────┐
│                      ▼         Kernel                   │
│  ┌─────────────────────────────────────────────────┐   │
│  │              Socket Layer (new)                  │   │
│  │  ┌─────────────┐  ┌─────────────┐               │   │
│  │  │SocketFile   │  │ UnixSocket  │               │   │
│  │  │ (VFS file)  │  │ (protocol)  │               │   │
│  │  └──────┬──────┘  └──────┬──────┘               │   │
│  │         │                │                       │   │
│  │         ▼                ▼                       │   │
│  │  ┌─────────────────────────────────────────┐    │   │
│  │  │         Socket Buffer (ring buffer)      │    │   │
│  │  └─────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
│                      │                                  │
│  ┌───────────────────┼─────────────────────────────┐   │
│  │                   ▼           VFS               │   │
│  │  ┌─────────────────────────────────────────┐    │   │
│  │  │  Socket Inode (S_IFSOCK) - for bind()   │    │   │
│  │  └─────────────────────────────────────────┘    │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

## Key Design Decisions

### 1. Socket as VFS File
Sockets are file descriptors, so they integrate with existing VFS infrastructure:
- `SocketFile` implements the `File` trait
- Read/write map to recv/send
- Close cleans up socket state

### 2. Unix Socket Types
Support both socket types:
- `SOCK_STREAM`: Connection-oriented, ordered, reliable (like TCP)
- `SOCK_DGRAM`: Connectionless datagrams (like UDP)

### 3. Address Namespaces
- **Filesystem**: `/path/to/socket` - creates inode in VFS
- **Abstract**: `\0name` - Linux-specific, no filesystem entry

### 4. Buffer Management
- Ring buffers for send/receive queues
- Configurable buffer sizes via `SO_SNDBUF`/`SO_RCVBUF`
- Default: 64KB per direction

## Dependencies

- Existing VFS infrastructure
- Pipe implementation (similar buffer management)
- epoll support (already implemented)

## Files to Create/Modify

### New Files
- `crates/kernel/syscall/src/socket/mod.rs` - Socket syscall implementations
- `crates/kernel/syscall/src/socket/unix.rs` - AF_UNIX protocol
- `crates/kernel/syscall/src/socket/types.rs` - Socket types and addresses
- `crates/kernel/vfs/src/socket.rs` - Socket inode support

### Modified Files
- `crates/kernel/syscall/src/lib.rs` - Add socket module and dispatch
- `crates/kernel/arch/*/src/lib.rs` - Add syscall numbers
- `crates/kernel/vfs/src/inode.rs` - Add S_IFSOCK file type

## References

- [Linux unix(7) man page](https://man7.org/linux/man-pages/man7/unix.7.html)
- [Linux socket(2) man page](https://man7.org/linux/man-pages/man2/socket.2.html)
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/ch04.html)
- [SCM_RIGHTS fd passing](https://man7.org/linux/man-pages/man7/unix.7.html)
