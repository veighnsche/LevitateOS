# Phase 2: Filesystem Sockets (bind/listen/accept/connect)

## Objective

Implement server-side socket operations (`bind`, `listen`, `accept`) and client-side `connect` to enable programs to communicate via filesystem socket paths like `/run/wayland-0`.

## Prerequisites

- Phase 1 complete (socket infrastructure, socketpair)
- VFS inode creation support

## Units of Work

### UoW 2.1: Socket Inode Type (S_IFSOCK)
**File:** `crates/kernel/vfs/src/inode.rs`

Add socket file type to VFS:

```rust
// File type constants (from Linux stat.h)
pub const S_IFSOCK: u32 = 0o140000;  // Socket

impl InodeType {
    pub const SOCKET: Self = Self(S_IFSOCK);
}

// In Inode struct or InodeData
pub struct SocketInodeData {
    /// The listening socket bound to this path
    pub socket: Weak<UnixSocket>,
}
```

**Acceptance:** VFS recognizes S_IFSOCK file type.

---

### UoW 2.2: sys_bind() Implementation
**File:** `crates/kernel/syscall/src/socket/unix.rs`

Bind a socket to a filesystem path:

```rust
/// Bind a socket to an address
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Pointer to sockaddr_un
/// * `addrlen` - Size of address structure
pub fn sys_bind(sockfd: i32, addr: usize, addrlen: u32) -> SyscallResult {
    let task = los_sched::current_task();

    // Get socket from fd
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Must be unbound
    if socket.state != SocketState::Unbound {
        return Err(EINVAL);
    }

    // Read sockaddr_un from user space
    let sockaddr = read_sockaddr_un(task.ttbr0, addr, addrlen)?;

    if sockaddr.sun_family != AF_UNIX as u16 {
        return Err(EAFNOSUPPORT);
    }

    // Extract path (null-terminated)
    let path = extract_socket_path(&sockaddr)?;

    // Check for abstract namespace (path starts with \0)
    if path.starts_with('\0') {
        return bind_abstract(socket, &path[1..]);
    }

    // Create socket inode in filesystem
    let parent_path = parent_dir(&path);
    let name = basename(&path);

    // Resolve parent directory
    let parent = los_vfs::resolve_path(&parent_path)?;

    // Check if path already exists
    if parent.lookup(name).is_ok() {
        return Err(EADDRINUSE);
    }

    // Create socket inode
    let socket_inode = create_socket_inode(Arc::downgrade(&socket.clone()))?;
    parent.create_child(name, socket_inode)?;

    // Update socket state
    socket.bound_path = Some(path.to_string());
    socket.state = SocketState::Bound;

    Ok(0)
}
```

**Acceptance:** Socket can be bound to path, path appears in filesystem.

---

### UoW 2.3: sys_listen() Implementation
**File:** `crates/kernel/syscall/src/socket/unix.rs`

Mark socket as listening for connections:

```rust
/// Listen for connections on a socket
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `backlog` - Maximum pending connections
pub fn sys_listen(sockfd: i32, backlog: i32) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Must be bound (for SOCK_STREAM)
    if socket.socket_type == SOCK_STREAM && socket.state != SocketState::Bound {
        return Err(EINVAL);
    }

    // Set backlog (clamped to reasonable limits)
    let backlog = backlog.clamp(1, 128) as usize;
    socket.max_backlog = backlog;

    // Transition to listening state
    socket.state = SocketState::Listening;

    // Register in global listener table for connect() to find
    register_listener(&socket.bound_path.as_ref().unwrap(), socket.clone())?;

    Ok(0)
}
```

**Acceptance:** Socket transitions to listening state.

---

### UoW 2.4: sys_accept() / sys_accept4() Implementation
**File:** `crates/kernel/syscall/src/socket/unix.rs`

Accept incoming connection:

```rust
/// Accept a connection on a listening socket
///
/// # Arguments
/// * `sockfd` - Listening socket fd
/// * `addr` - Optional: where to store peer address
/// * `addrlen` - Optional: address length
/// * `flags` - SOCK_NONBLOCK, SOCK_CLOEXEC (accept4 only)
pub fn sys_accept4(
    sockfd: i32,
    addr: usize,
    addrlen: usize,
    flags: i32,
) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Must be listening
    if socket.state != SocketState::Listening {
        return Err(EINVAL);
    }

    // Try to get pending connection
    let pending = {
        let mut backlog = socket.backlog.lock();
        backlog.pop_front()
    };

    let client_socket = match pending {
        Some(sock) => sock,
        None => {
            if socket.is_nonblock() || flags & SOCK_NONBLOCK != 0 {
                return Err(EAGAIN);
            }
            // Block waiting for connection
            // TODO: proper blocking with wait queue
            return Err(EAGAIN);
        }
    };

    // Create server-side socket connected to client
    let server_socket = Arc::new(UnixSocket::new(socket.socket_type));
    server_socket.connect_to_peer(client_socket.clone());
    client_socket.connect_to_peer(server_socket.clone());

    server_socket.state = SocketState::Connected;
    client_socket.state = SocketState::Connected;

    // Wake up client if it was blocking on connect()
    client_socket.wake_connect_waiters();

    // Create file and add to fd table
    let file = Arc::new(SocketFile { socket: server_socket });
    let cloexec = flags & SOCK_CLOEXEC != 0;
    let fd = add_to_fd_table(file, cloexec)?;

    // Write peer address if requested
    if addr != 0 && addrlen != 0 {
        write_peer_address(addr, addrlen, &client_socket)?;
    }

    Ok(fd as i64)
}

pub fn sys_accept(sockfd: i32, addr: usize, addrlen: usize) -> SyscallResult {
    sys_accept4(sockfd, addr, addrlen, 0)
}
```

**Acceptance:** accept() returns new connected socket fd.

---

### UoW 2.5: sys_connect() Implementation
**File:** `crates/kernel/syscall/src/socket/unix.rs`

Connect to a listening socket:

```rust
/// Connect to a socket
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `addr` - Server address (sockaddr_un)
/// * `addrlen` - Address length
pub fn sys_connect(sockfd: i32, addr: usize, addrlen: u32) -> SyscallResult {
    let task = los_sched::current_task();
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Check state
    match socket.state {
        SocketState::Connected => return Err(EISCONN),
        SocketState::Connecting => return Err(EALREADY),
        SocketState::Listening => return Err(EINVAL),
        _ => {}
    }

    // Read target address
    let sockaddr = read_sockaddr_un(task.ttbr0, addr, addrlen)?;
    let path = extract_socket_path(&sockaddr)?;

    // Find listening socket
    let listener = if path.starts_with('\0') {
        find_abstract_listener(&path[1..])?
    } else {
        find_path_listener(&path)?
    };

    // Check listener has room in backlog
    {
        let mut backlog = listener.backlog.lock();
        if backlog.len() >= listener.max_backlog {
            return Err(ECONNREFUSED);
        }

        // Add ourselves to the backlog
        socket.state = SocketState::Connecting;
        backlog.push_back(socket.clone());
    }

    // Wake up listener if it's in accept()
    listener.wake_accept_waiters();

    // For non-blocking, return EINPROGRESS
    if socket.is_nonblock() {
        return Err(EINPROGRESS);
    }

    // Block until connected
    // TODO: proper blocking
    // For now, spin-wait (temporary)
    while socket.state == SocketState::Connecting {
        los_sched::yield_now();
    }

    if socket.state != SocketState::Connected {
        return Err(ECONNREFUSED);
    }

    Ok(0)
}
```

**Acceptance:** Client can connect to server socket.

---

### UoW 2.6: Listener Registry
**File:** `crates/kernel/syscall/src/socket/unix.rs`

Global registry for finding listening sockets by path:

```rust
use alloc::collections::BTreeMap;
use los_utils::SpinLock;

/// Global registry of listening sockets by path
static LISTENERS: SpinLock<BTreeMap<String, Weak<UnixSocket>>> =
    SpinLock::new(BTreeMap::new());

/// Abstract namespace listeners (paths starting with \0)
static ABSTRACT_LISTENERS: SpinLock<BTreeMap<String, Weak<UnixSocket>>> =
    SpinLock::new(BTreeMap::new());

pub fn register_listener(path: &str, socket: Arc<UnixSocket>) -> Result<(), u32> {
    let mut listeners = LISTENERS.lock();
    if listeners.contains_key(path) {
        return Err(EADDRINUSE);
    }
    listeners.insert(path.to_string(), Arc::downgrade(&socket));
    Ok(())
}

pub fn unregister_listener(path: &str) {
    LISTENERS.lock().remove(path);
}

pub fn find_path_listener(path: &str) -> Result<Arc<UnixSocket>, u32> {
    let listeners = LISTENERS.lock();
    listeners
        .get(path)
        .and_then(|weak| weak.upgrade())
        .ok_or(ECONNREFUSED)
}
```

**Acceptance:** Listeners can be registered and found by path.

---

### UoW 2.7: Socket Cleanup on Close
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Clean up socket resources when closed:

```rust
impl Drop for UnixSocket {
    fn drop(&mut self) {
        // Unregister from listener table
        if let Some(path) = &self.bound_path {
            unregister_listener(path);

            // Remove socket file from filesystem
            if !path.starts_with('\0') {
                let _ = los_vfs::unlink(path);
            }
        }

        // Notify peer of disconnect
        if let Some(peer) = &self.peer {
            peer.state = SocketState::Disconnected;
            peer.wake_all_waiters();
        }
    }
}
```

**Acceptance:** Socket cleanup removes listener registration and notifies peer.

---

### UoW 2.8: sys_shutdown() Implementation
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Shut down part of a full-duplex connection:

```rust
/// Shut down part of a socket connection
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `how` - SHUT_RD, SHUT_WR, or SHUT_RDWR
pub fn sys_shutdown(sockfd: i32, how: i32) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    match how {
        SHUT_RD => socket.shutdown_read(),
        SHUT_WR => socket.shutdown_write(),
        SHUT_RDWR => {
            socket.shutdown_read();
            socket.shutdown_write();
        }
        _ => return Err(EINVAL),
    }

    Ok(0)
}
```

**Acceptance:** shutdown() prevents further reads/writes.

---

### UoW 2.9: getsockname/getpeername
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Get local and peer socket addresses:

```rust
pub fn sys_getsockname(sockfd: i32, addr: usize, addrlen: usize) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    if let Some(path) = &socket.bound_path {
        write_sockaddr_un(addr, addrlen, path)?;
    } else {
        // Unnamed socket
        write_sockaddr_un(addr, addrlen, "")?;
    }

    Ok(0)
}

pub fn sys_getpeername(sockfd: i32, addr: usize, addrlen: usize) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    let peer = socket.peer.as_ref().ok_or(ENOTCONN)?;

    if let Some(path) = &peer.bound_path {
        write_sockaddr_un(addr, addrlen, path)?;
    } else {
        write_sockaddr_un(addr, addrlen, "")?;
    }

    Ok(0)
}
```

**Acceptance:** Can query socket and peer addresses.

---

### UoW 2.10: Syscall Dispatch for Phase 2
**File:** `crates/kernel/syscall/src/lib.rs`

Add remaining socket syscall dispatches:

```rust
Some(SyscallNumber::Bind) => socket::sys_bind(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as u32,
),
Some(SyscallNumber::Listen) => socket::sys_listen(
    frame.arg0() as i32,
    frame.arg1() as i32,
),
Some(SyscallNumber::Accept) => socket::sys_accept(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
),
Some(SyscallNumber::Accept4) => socket::sys_accept4(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
    frame.arg3() as i32,
),
Some(SyscallNumber::Connect) => socket::sys_connect(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as u32,
),
Some(SyscallNumber::Shutdown) => socket::sys_shutdown(
    frame.arg0() as i32,
    frame.arg1() as i32,
),
Some(SyscallNumber::Getsockname) => socket::sys_getsockname(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
),
Some(SyscallNumber::Getpeername) => socket::sys_getpeername(
    frame.arg0() as i32,
    frame.arg1() as usize,
    frame.arg2() as usize,
),
```

---

## Testing

### Test 1: Basic Server-Client
```c
// Server
int server = socket(AF_UNIX, SOCK_STREAM, 0);
struct sockaddr_un addr = { .sun_family = AF_UNIX };
strcpy(addr.sun_path, "/tmp/test.sock");
bind(server, &addr, sizeof(addr));
listen(server, 5);

int client_fd = accept(server, NULL, NULL);
char buf[100];
read(client_fd, buf, 100);
```

```c
// Client
int client = socket(AF_UNIX, SOCK_STREAM, 0);
struct sockaddr_un addr = { .sun_family = AF_UNIX };
strcpy(addr.sun_path, "/tmp/test.sock");
connect(client, &addr, sizeof(addr));
write(client, "hello", 5);
```

### Test 2: Socket file exists
```c
bind(sock, "/tmp/test.sock", ...);
struct stat st;
assert(stat("/tmp/test.sock", &st) == 0);
assert(S_ISSOCK(st.st_mode));
```

### Test 3: Address in use
```c
bind(sock1, "/tmp/test.sock", ...);
assert(bind(sock2, "/tmp/test.sock", ...) == -1);
assert(errno == EADDRINUSE);
```

---

## Verification Checklist

- [ ] bind() creates socket file in filesystem
- [ ] listen() transitions socket to listening state
- [ ] accept() blocks until connection arrives
- [ ] connect() finds and connects to listener
- [ ] Data flows between connected sockets
- [ ] Socket file has S_IFSOCK mode
- [ ] Closing listener unregisters and removes file
- [ ] EADDRINUSE on duplicate bind
- [ ] ECONNREFUSED when no listener
- [ ] Abstract namespace works (\0path)
