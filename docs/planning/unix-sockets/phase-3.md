# Phase 3: Advanced Features (FD Passing, Credentials)

## Objective

Implement `sendmsg()`/`recvmsg()` with ancillary data support for:
- **SCM_RIGHTS**: File descriptor passing (critical for Wayland)
- **SCM_CREDENTIALS**: Process credential passing

This is what makes Wayland possible - clients pass shared memory buffer fds to the compositor.

## Prerequisites

- Phase 1 & 2 complete
- Working socket infrastructure

## Background: How FD Passing Works

When a process sends a file descriptor via `sendmsg()`:
1. Sender puts fd number in `SCM_RIGHTS` control message
2. Kernel extracts the underlying `File` object from sender's fd table
3. Kernel installs a **new fd** pointing to same `File` in receiver's fd table
4. Receiver gets a different fd number but same underlying file

```
Sender Process              Kernel                  Receiver Process
┌─────────────┐        ┌─────────────┐           ┌─────────────┐
│ fd 5 ───────┼───────►│ File object │◄──────────┼─── fd 9     │
│             │        │  (shared)   │           │             │
└─────────────┘        └─────────────┘           └─────────────┘
```

## Units of Work

### UoW 3.1: Control Message Types
**File:** `crates/kernel/syscall/src/socket/types.rs`

Define ancillary data structures:

```rust
// Control message level
pub const SOL_SOCKET: i32 = 1;

// Control message types
pub const SCM_RIGHTS: i32 = 0x01;      // File descriptors
pub const SCM_CREDENTIALS: i32 = 0x02; // Process credentials

/// Control message header (Linux cmsghdr)
#[repr(C)]
pub struct Cmsghdr {
    pub cmsg_len: usize,   // Length including header
    pub cmsg_level: i32,   // Originating protocol (SOL_SOCKET)
    pub cmsg_type: i32,    // Protocol-specific type (SCM_RIGHTS, etc.)
    // Followed by: data (variable length, aligned)
}

/// Message header for sendmsg/recvmsg
#[repr(C)]
pub struct Msghdr {
    pub msg_name: usize,       // Optional address
    pub msg_namelen: u32,      // Address length
    pub msg_iov: usize,        // Scatter/gather array (iovec*)
    pub msg_iovlen: usize,     // Number of elements in msg_iov
    pub msg_control: usize,    // Ancillary data buffer
    pub msg_controllen: usize, // Ancillary data length
    pub msg_flags: i32,        // Flags on received message
}

/// Credentials structure for SCM_CREDENTIALS
#[repr(C)]
pub struct Ucred {
    pub pid: i32,
    pub uid: u32,
    pub gid: u32,
}

// Helper macros for control message manipulation
pub const fn cmsg_align(len: usize) -> usize {
    (len + core::mem::size_of::<usize>() - 1) & !(core::mem::size_of::<usize>() - 1)
}

pub const fn cmsg_space(len: usize) -> usize {
    cmsg_align(core::mem::size_of::<Cmsghdr>()) + cmsg_align(len)
}

pub const fn cmsg_len(len: usize) -> usize {
    cmsg_align(core::mem::size_of::<Cmsghdr>()) + len
}
```

**Acceptance:** Control message structures match Linux ABI.

---

### UoW 3.2: sys_sendmsg() Implementation
**File:** `crates/kernel/syscall/src/socket/msg.rs`

```rust
/// Send a message on a socket with optional control data
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `msg` - Message header (msghdr pointer)
/// * `flags` - Send flags
pub fn sys_sendmsg(sockfd: i32, msg: usize, flags: i32) -> SyscallResult {
    let task = los_sched::current_task();
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Read msghdr from user space
    let msghdr: Msghdr = read_struct_from_user(task.ttbr0, msg)?;

    // Read iovec array for data
    let data = read_iovec_data(task.ttbr0, msghdr.msg_iov, msghdr.msg_iovlen)?;

    // Process control messages (ancillary data)
    let mut fds_to_send: Vec<Arc<dyn File>> = Vec::new();

    if msghdr.msg_control != 0 && msghdr.msg_controllen > 0 {
        let control_buf = read_user_buffer(task.ttbr0, msghdr.msg_control, msghdr.msg_controllen)?;

        // Parse control messages
        let mut offset = 0;
        while offset + core::mem::size_of::<Cmsghdr>() <= control_buf.len() {
            let cmsg: Cmsghdr = unsafe {
                core::ptr::read_unaligned(control_buf[offset..].as_ptr() as *const Cmsghdr)
            };

            if cmsg.cmsg_len < core::mem::size_of::<Cmsghdr>() {
                break;
            }

            let data_offset = offset + cmsg_align(core::mem::size_of::<Cmsghdr>());
            let data_len = cmsg.cmsg_len - core::mem::size_of::<Cmsghdr>();

            if cmsg.cmsg_level == SOL_SOCKET && cmsg.cmsg_type == SCM_RIGHTS {
                // Extract file descriptors to send
                let num_fds = data_len / core::mem::size_of::<i32>();
                for i in 0..num_fds {
                    let fd_offset = data_offset + i * core::mem::size_of::<i32>();
                    let fd: i32 = unsafe {
                        core::ptr::read_unaligned(control_buf[fd_offset..].as_ptr() as *const i32)
                    };

                    // Get the File object from sender's fd table
                    let file = get_vfs_file(fd as usize)?;
                    fds_to_send.push(file);
                }
            }

            offset += cmsg_align(cmsg.cmsg_len);
        }
    }

    // Send data and fds to peer
    socket.send_with_fds(&data, &fds_to_send, flags)
}
```

**Acceptance:** sendmsg() can send data with file descriptors attached.

---

### UoW 3.3: sys_recvmsg() Implementation
**File:** `crates/kernel/syscall/src/socket/msg.rs`

```rust
/// Receive a message from a socket with optional control data
///
/// # Arguments
/// * `sockfd` - Socket file descriptor
/// * `msg` - Message header (msghdr pointer)
/// * `flags` - Receive flags
pub fn sys_recvmsg(sockfd: i32, msg: usize, flags: i32) -> SyscallResult {
    let task = los_sched::current_task();
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    // Read msghdr from user space
    let mut msghdr: Msghdr = read_struct_from_user(task.ttbr0, msg)?;

    // Receive data and any pending fds
    let (data, received_fds) = socket.recv_with_fds(
        msghdr.msg_iovlen * 4096, // Max size hint
        flags,
    )?;

    // Write data to iovec
    let bytes_written = write_to_iovec(
        task.ttbr0,
        msghdr.msg_iov,
        msghdr.msg_iovlen,
        &data,
    )?;

    // Process received file descriptors
    if !received_fds.is_empty() && msghdr.msg_control != 0 {
        let mut control_buf = vec![0u8; msghdr.msg_controllen];
        let mut offset = 0;

        // Build SCM_RIGHTS control message
        let cmsg_data_len = received_fds.len() * core::mem::size_of::<i32>();
        let cmsg_total_len = cmsg_len(cmsg_data_len);

        if offset + cmsg_space(cmsg_data_len) <= control_buf.len() {
            // Write cmsghdr
            let cmsg = Cmsghdr {
                cmsg_len: cmsg_total_len,
                cmsg_level: SOL_SOCKET,
                cmsg_type: SCM_RIGHTS,
            };
            unsafe {
                core::ptr::write_unaligned(
                    control_buf[offset..].as_mut_ptr() as *mut Cmsghdr,
                    cmsg,
                );
            }

            // Install fds in receiver's table and write fd numbers
            let fd_offset = offset + cmsg_align(core::mem::size_of::<Cmsghdr>());
            for (i, file) in received_fds.iter().enumerate() {
                let new_fd = install_fd_in_table(file.clone(), false)?;
                unsafe {
                    core::ptr::write_unaligned(
                        control_buf[fd_offset + i * 4..].as_mut_ptr() as *mut i32,
                        new_fd as i32,
                    );
                }
            }

            offset += cmsg_space(cmsg_data_len);
        }

        // Write control buffer back to user
        write_user_buffer(task.ttbr0, msghdr.msg_control, &control_buf[..offset])?;

        // Update controllen with actual size
        msghdr.msg_controllen = offset;
    } else {
        msghdr.msg_controllen = 0;
    }

    // Write updated msghdr back
    write_struct_to_user(task.ttbr0, msg, &msghdr)?;

    Ok(bytes_written as i64)
}
```

**Acceptance:** recvmsg() receives data and installs passed file descriptors.

---

### UoW 3.4: Socket FD Queue
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Add fd queue to socket structure:

```rust
pub struct UnixSocket {
    // ... existing fields ...

    /// Queue of file descriptors waiting to be received
    /// Each entry is a list of fds that came with one message
    pub pending_fds: SpinLock<VecDeque<Vec<Arc<dyn File>>>>,
}

impl UnixSocket {
    /// Send data with attached file descriptors
    pub fn send_with_fds(
        &self,
        data: &[u8],
        fds: &[Arc<dyn File>],
        flags: i32,
    ) -> SyscallResult {
        let peer = self.peer.as_ref().ok_or(ENOTCONN)?;

        // Queue the fds first (associated with this message)
        if !fds.is_empty() {
            let mut pending = peer.pending_fds.lock();
            pending.push_back(fds.to_vec());
        }

        // Then send the data
        let mut recv_buf = peer.recv_buf.lock();
        let written = recv_buf.write(data);

        if written == 0 && !data.is_empty() {
            // Remove queued fds if data couldn't be sent
            if !fds.is_empty() {
                peer.pending_fds.lock().pop_back();
            }
            if self.is_nonblock() {
                return Err(EAGAIN);
            }
        }

        Ok(written as i64)
    }

    /// Receive data with any attached file descriptors
    pub fn recv_with_fds(
        &self,
        max_len: usize,
        flags: i32,
    ) -> Result<(Vec<u8>, Vec<Arc<dyn File>>), u32> {
        let mut data = vec![0u8; max_len];

        let mut recv_buf = self.recv_buf.lock();
        let read = recv_buf.read(&mut data);
        data.truncate(read);

        // Get any pending fds for this message
        let fds = self.pending_fds.lock().pop_front().unwrap_or_default();

        if read == 0 && fds.is_empty() {
            if self.peer.is_none() {
                return Ok((data, fds)); // EOF
            }
            if self.is_nonblock() {
                return Err(EAGAIN);
            }
        }

        Ok((data, fds))
    }
}
```

**Acceptance:** File descriptors can be queued and retrieved with messages.

---

### UoW 3.5: SCM_CREDENTIALS Support
**File:** `crates/kernel/syscall/src/socket/msg.rs`

Add credential passing (optional but useful):

```rust
/// Send credentials in control message
fn send_credentials(socket: &UnixSocket, cmsg_buf: &mut [u8], offset: &mut usize) {
    let task = los_sched::current_task();

    let cred = Ucred {
        pid: task.id.0 as i32,
        uid: 0, // LevitateOS is single-user
        gid: 0,
    };

    let cmsg = Cmsghdr {
        cmsg_len: cmsg_len(core::mem::size_of::<Ucred>()),
        cmsg_level: SOL_SOCKET,
        cmsg_type: SCM_CREDENTIALS,
    };

    // Write header and data...
}
```

**Acceptance:** Credentials can be passed with messages.

---

### UoW 3.6: Socket Options (getsockopt/setsockopt)
**File:** `crates/kernel/syscall/src/socket/mod.rs`

Basic socket options:

```rust
// Socket options
pub const SO_REUSEADDR: i32 = 2;
pub const SO_SNDBUF: i32 = 7;
pub const SO_RCVBUF: i32 = 8;
pub const SO_PASSCRED: i32 = 16;
pub const SO_PEERCRED: i32 = 17;

pub fn sys_setsockopt(
    sockfd: i32,
    level: i32,
    optname: i32,
    optval: usize,
    optlen: u32,
) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    match (level, optname) {
        (SOL_SOCKET, SO_PASSCRED) => {
            // Enable/disable credential passing
            let val: i32 = read_struct_from_user(los_sched::current_task().ttbr0, optval)?;
            socket.set_passcred(val != 0);
            Ok(0)
        }
        (SOL_SOCKET, SO_SNDBUF) => {
            // Set send buffer size (ignored for now)
            Ok(0)
        }
        (SOL_SOCKET, SO_RCVBUF) => {
            // Set receive buffer size (ignored for now)
            Ok(0)
        }
        _ => {
            log::warn!("[SOCKET] Unknown socket option: level={}, opt={}", level, optname);
            Ok(0) // Silently succeed for compatibility
        }
    }
}

pub fn sys_getsockopt(
    sockfd: i32,
    level: i32,
    optname: i32,
    optval: usize,
    optlen: usize,
) -> SyscallResult {
    let socket_file = get_socket_file(sockfd)?;
    let socket = &socket_file.socket;

    match (level, optname) {
        (SOL_SOCKET, SO_PEERCRED) => {
            // Get peer credentials
            let cred = Ucred {
                pid: socket.peer.as_ref().map(|p| p.peer_pid).unwrap_or(0),
                uid: 0,
                gid: 0,
            };
            write_struct_to_user(los_sched::current_task().ttbr0, optval, &cred)?;
            Ok(0)
        }
        _ => {
            log::warn!("[SOCKET] Unknown getsockopt: level={}, opt={}", level, optname);
            Err(ENOPROTOOPT)
        }
    }
}
```

**Acceptance:** Basic socket options work.

---

## Testing

### Test 1: FD Passing
```c
int sv[2];
socketpair(AF_UNIX, SOCK_STREAM, 0, sv);

// Open a file and send its fd
int fd = open("/tmp/test.txt", O_RDWR | O_CREAT, 0644);
write(fd, "hello", 5);
lseek(fd, 0, SEEK_SET);

// Send fd via SCM_RIGHTS
struct msghdr msg = {0};
struct iovec iov = { .iov_base = "x", .iov_len = 1 };
char cmsgbuf[CMSG_SPACE(sizeof(int))];

msg.msg_iov = &iov;
msg.msg_iovlen = 1;
msg.msg_control = cmsgbuf;
msg.msg_controllen = sizeof(cmsgbuf);

struct cmsghdr *cmsg = CMSG_FIRSTHDR(&msg);
cmsg->cmsg_level = SOL_SOCKET;
cmsg->cmsg_type = SCM_RIGHTS;
cmsg->cmsg_len = CMSG_LEN(sizeof(int));
*(int*)CMSG_DATA(cmsg) = fd;

sendmsg(sv[0], &msg, 0);

// Receive fd on other end
struct msghdr rmsg = {0};
char rbuf[1];
struct iovec riov = { .iov_base = rbuf, .iov_len = 1 };
char rcmsgbuf[CMSG_SPACE(sizeof(int))];

rmsg.msg_iov = &riov;
rmsg.msg_iovlen = 1;
rmsg.msg_control = rcmsgbuf;
rmsg.msg_controllen = sizeof(rcmsgbuf);

recvmsg(sv[1], &rmsg, 0);

struct cmsghdr *rcmsg = CMSG_FIRSTHDR(&rmsg);
int received_fd = *(int*)CMSG_DATA(rcmsg);

// Read from received fd - should get "hello"
char buf[10];
assert(read(received_fd, buf, 10) == 5);
assert(memcmp(buf, "hello", 5) == 0);
```

### Test 2: Multiple FDs
```c
// Send 3 fds at once
int fds[3] = { fd1, fd2, fd3 };
// ... build cmsg with 3 fds ...
sendmsg(sv[0], &msg, 0);

// Receive all 3
recvmsg(sv[1], &rmsg, 0);
// Check all 3 fds were received
```

---

## Verification Checklist

- [ ] sendmsg() with SCM_RIGHTS sends file descriptors
- [ ] recvmsg() receives and installs fds in new fd table slots
- [ ] Multiple fds can be sent in one message
- [ ] FD numbers are different in receiver (but same underlying file)
- [ ] Closing original fd doesn't affect received fd
- [ ] SCM_CREDENTIALS passes process info
- [ ] getsockopt(SO_PEERCRED) returns peer credentials
- [ ] Works with both socketpair and filesystem sockets
