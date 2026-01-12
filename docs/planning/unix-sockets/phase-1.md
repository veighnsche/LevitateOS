# Phase 1: Socket Infrastructure & Socketpair

## Objective

Build the core socket infrastructure and implement `socketpair()` properly (replacing current pipe-based stub).

## Prerequisites

- Existing pipe implementation (for reference)
- VFS file descriptor infrastructure

## Units of Work

### UoW 1.1: Socket Types and Constants
**File:** `crates/kernel/syscall/src/socket/types.rs`

Define socket-related constants from Linux ABI:

```rust
// Address families
pub const AF_UNIX: i32 = 1;
pub const AF_LOCAL: i32 = AF_UNIX;

// Socket types
pub const SOCK_STREAM: i32 = 1;
pub const SOCK_DGRAM: i32 = 2;
pub const SOCK_SEQPACKET: i32 = 5;

// Socket type flags (can be OR'd with type)
pub const SOCK_NONBLOCK: i32 = 0o4000;
pub const SOCK_CLOEXEC: i32 = 0o2000000;

// Shutdown how
pub const SHUT_RD: i32 = 0;
pub const SHUT_WR: i32 = 1;
pub const SHUT_RDWR: i32 = 2;

// Unix socket address
#[repr(C)]
pub struct SockaddrUn {
    pub sun_family: u16,      // AF_UNIX
    pub sun_path: [u8; 108],  // Path name
}
```

**Acceptance:** Constants compile and match Linux values.

---

### UoW 1.2: Socket State Structure
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Define the core socket state:

```rust
pub struct UnixSocket {
    /// Socket type (SOCK_STREAM or SOCK_DGRAM)
    pub socket_type: i32,

    /// Socket state
    pub state: SocketState,

    /// Receive buffer
    pub recv_buf: SpinLock<RingBuffer<u8>>,

    /// Send buffer (for connected sockets, points to peer's recv_buf)
    pub peer: Option<Arc<UnixSocket>>,

    /// Bound address (if any)
    pub bound_path: Option<String>,

    /// Pending connections (for listening sockets)
    pub backlog: SpinLock<VecDeque<Arc<UnixSocket>>>,

    /// Max backlog size
    pub max_backlog: usize,

    /// Socket options
    pub flags: AtomicU32,
}

pub enum SocketState {
    Unbound,
    Bound,
    Listening,
    Connecting,
    Connected,
    Disconnected,
}
```

**Acceptance:** Socket structure compiles with all necessary fields.

---

### UoW 1.3: SocketFile VFS Wrapper
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Create a VFS-compatible file wrapper:

```rust
pub struct SocketFile {
    pub socket: Arc<UnixSocket>,
}

impl File for SocketFile {
    fn read(&self, buf: &mut [u8]) -> Result<usize, u32> {
        // Delegate to socket recv
        self.socket.recv(buf, 0)
    }

    fn write(&self, buf: &[u8]) -> Result<usize, u32> {
        // Delegate to socket send
        self.socket.send(buf, 0)
    }

    fn poll(&self) -> PollFlags {
        // Check readable/writable state
    }

    // ... other File trait methods
}
```

**Acceptance:** SocketFile implements File trait, can be added to fd table.

---

### UoW 1.4: Socket Syscall Numbers
**Files:** `crates/kernel/arch/{aarch64,x86_64}/src/lib.rs`

Add syscall numbers for both architectures:

| Syscall | x86_64 | aarch64 |
|---------|--------|---------|
| socket | 41 | 198 |
| socketpair | 53 | 199 |
| bind | 49 | 200 |
| listen | 50 | 201 |
| accept | 43 | 202 |
| accept4 | 288 | 242 |
| connect | 42 | 203 |
| sendto | 44 | 206 |
| recvfrom | 45 | 207 |
| sendmsg | 46 | 211 |
| recvmsg | 47 | 212 |
| shutdown | 48 | 210 |
| getsockopt | 55 | 209 |
| setsockopt | 54 | 208 |
| getsockname | 51 | 204 |
| getpeername | 52 | 205 |

**Acceptance:** All syscall numbers added to both arch files with from_u64 cases.

---

### UoW 1.5: sys_socket() Implementation
**File:** `crates/kernel/syscall/src/socket/mod.rs`

```rust
/// Create a socket
///
/// # Arguments
/// * `domain` - Address family (AF_UNIX)
/// * `socket_type` - Socket type (SOCK_STREAM, SOCK_DGRAM) + flags
/// * `protocol` - Protocol (0 for default)
pub fn sys_socket(domain: i32, socket_type: i32, protocol: i32) -> SyscallResult {
    // Only support AF_UNIX for now
    if domain != AF_UNIX {
        return Err(EAFNOSUPPORT);
    }

    // Extract type and flags
    let base_type = socket_type & 0xFF;
    let flags = socket_type & !0xFF;

    if base_type != SOCK_STREAM && base_type != SOCK_DGRAM {
        return Err(ESOCKTNOSUPPORT);
    }

    // Create socket
    let socket = Arc::new(UnixSocket::new(base_type));

    // Apply flags
    if flags & SOCK_NONBLOCK != 0 {
        socket.set_nonblock(true);
    }

    // Wrap in SocketFile and add to fd table
    let file = Arc::new(SocketFile { socket });
    let fd = add_to_fd_table(file, flags & SOCK_CLOEXEC != 0)?;

    Ok(fd as i64)
}
```

**Acceptance:** `socket(AF_UNIX, SOCK_STREAM, 0)` returns valid fd.

---

### UoW 1.6: sys_socketpair() Implementation
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Replace the current stub with real implementation:

```rust
/// Create a pair of connected sockets
///
/// # Arguments
/// * `domain` - Address family (AF_UNIX)
/// * `socket_type` - Socket type + flags
/// * `protocol` - Protocol (0)
/// * `sv` - User pointer to int[2] for returned fds
pub fn sys_socketpair(
    domain: i32,
    socket_type: i32,
    protocol: i32,
    sv: usize,
) -> SyscallResult {
    if domain != AF_UNIX {
        return Err(EAFNOSUPPORT);
    }

    let base_type = socket_type & 0xFF;
    let flags = socket_type & !0xFF;

    // Create two sockets
    let sock1 = Arc::new(UnixSocket::new(base_type));
    let sock2 = Arc::new(UnixSocket::new(base_type));

    // Connect them to each other
    sock1.connect_to_peer(sock2.clone());
    sock2.connect_to_peer(sock1.clone());

    // Mark as connected
    sock1.state = SocketState::Connected;
    sock2.state = SocketState::Connected;

    // Create file wrappers
    let file1 = Arc::new(SocketFile { socket: sock1 });
    let file2 = Arc::new(SocketFile { socket: sock2 });

    // Add to fd table
    let cloexec = flags & SOCK_CLOEXEC != 0;
    let fd1 = add_to_fd_table(file1, cloexec)?;
    let fd2 = add_to_fd_table(file2, cloexec)?;

    // Write fds to user space
    write_user_i32_pair(sv, fd1, fd2)?;

    Ok(0)
}
```

**Acceptance:** `socketpair()` creates two connected sockets that can send/recv data.

---

### UoW 1.7: Basic send/recv for Connected Sockets
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Implement data transfer for connected socket pairs:

```rust
impl UnixSocket {
    /// Send data to connected peer
    pub fn send(&self, buf: &[u8], flags: i32) -> Result<usize, u32> {
        let peer = self.peer.as_ref().ok_or(ENOTCONN)?;

        // Write to peer's receive buffer
        let mut recv_buf = peer.recv_buf.lock();
        let written = recv_buf.write(buf);

        if written == 0 && !buf.is_empty() {
            if self.is_nonblock() {
                return Err(EAGAIN);
            }
            // Would block - for now just return 0
            // TODO: proper blocking
        }

        Ok(written)
    }

    /// Receive data from connected peer
    pub fn recv(&self, buf: &mut [u8], flags: i32) -> Result<usize, u32> {
        let mut recv_buf = self.recv_buf.lock();
        let read = recv_buf.read(buf);

        if read == 0 {
            // Check if peer disconnected
            if self.peer.is_none() || self.state == SocketState::Disconnected {
                return Ok(0); // EOF
            }
            if self.is_nonblock() {
                return Err(EAGAIN);
            }
        }

        Ok(read)
    }
}
```

**Acceptance:** Data written to one socket can be read from the other.

---

### UoW 1.8: Syscall Dispatch Integration
**File:** `crates/kernel/syscall/src/lib.rs`

Add dispatch entries for socket syscalls:

```rust
Some(SyscallNumber::Socket) => socket::sys_socket(
    frame.arg0() as i32,
    frame.arg1() as i32,
    frame.arg2() as i32,
),
Some(SyscallNumber::Socketpair) => socket::sys_socketpair(
    frame.arg0() as i32,
    frame.arg1() as i32,
    frame.arg2() as i32,
    frame.arg3() as usize,
),
// ... more to come in phase 2
```

**Acceptance:** Syscalls dispatch correctly to socket module.

---

## Testing

### Test 1: socketpair basic functionality
```c
int sv[2];
assert(socketpair(AF_UNIX, SOCK_STREAM, 0, sv) == 0);
assert(sv[0] >= 0 && sv[1] >= 0);

char buf[] = "hello";
assert(write(sv[0], buf, 5) == 5);

char recv[10];
assert(read(sv[1], recv, 10) == 5);
assert(memcmp(recv, "hello", 5) == 0);
```

### Test 2: Bidirectional communication
```c
// Write from both ends, read from both ends
write(sv[0], "A", 1);
write(sv[1], "B", 1);
read(sv[0], buf, 1);  // Should get "B"
read(sv[1], buf, 1);  // Should get "A"
```

### Test 3: Close propagation
```c
close(sv[0]);
// Read from sv[1] should return 0 (EOF)
assert(read(sv[1], buf, 10) == 0);
```

---

## Verification Checklist

- [ ] Socket types and constants match Linux ABI
- [ ] `socket(AF_UNIX, SOCK_STREAM, 0)` returns valid fd
- [ ] `socket(AF_INET, ...)` returns EAFNOSUPPORT (not supported yet)
- [ ] `socketpair()` creates connected pair
- [ ] Data flows bidirectionally between socket pair
- [ ] Close one end causes EOF on the other
- [ ] SOCK_NONBLOCK flag works
- [ ] SOCK_CLOEXEC flag works
- [ ] Builds on both x86_64 and aarch64
