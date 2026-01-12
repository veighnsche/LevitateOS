# Phase 4: Shared Memory Support

## Objective

Implement shared memory primitives needed for Wayland buffer sharing:
- `MAP_SHARED` flag for mmap
- `memfd_create()` syscall

## Why This Matters for Wayland

Wayland clients share buffers with the compositor efficiently via shared memory:

```
Client Process                      Compositor Process
┌─────────────────┐                ┌─────────────────┐
│                 │                │                 │
│  ┌───────────┐  │   sendmsg()    │  ┌───────────┐  │
│  │ Buffer    │  │  (SCM_RIGHTS)  │  │ Buffer    │  │
│  │ (mmap'd)  │──┼───────────────►│  │ (mmap'd)  │  │
│  └─────┬─────┘  │                │  └─────┬─────┘  │
│        │        │                │        │        │
└────────┼────────┘                └────────┼────────┘
         │                                  │
         ▼                                  ▼
    ┌─────────────────────────────────────────┐
    │         Shared Physical Pages           │
    │  (both processes see same memory)       │
    └─────────────────────────────────────────┘
```

## Current State

| Feature | Status |
|---------|--------|
| `mmap(MAP_PRIVATE)` | Working |
| `mmap(MAP_SHARED)` | Not implemented |
| `memfd_create()` | Not implemented |
| `shm_open()` | Not implemented (libc, not syscall) |

## Units of Work

### UoW 4.1: MAP_SHARED Support in mmap
**File:** `crates/kernel/syscall/src/mm.rs`

Current mmap always creates private mappings. Add shared mapping support:

```rust
pub const MAP_SHARED: u32 = 0x01;
pub const MAP_PRIVATE: u32 = 0x02;
pub const MAP_ANONYMOUS: u32 = 0x20;

pub fn sys_mmap(
    addr: usize,
    len: usize,
    prot: u32,
    flags: u32,
    fd: i32,
    offset: usize,
) -> SyscallResult {
    // ... existing validation ...

    let is_shared = flags & MAP_SHARED != 0;
    let is_anonymous = flags & MAP_ANONYMOUS != 0;

    if is_shared && !is_anonymous {
        // File-backed shared mapping
        return mmap_shared_file(addr, len, prot, fd, offset);
    } else if is_shared && is_anonymous {
        // Anonymous shared mapping (rare but valid)
        return mmap_shared_anonymous(addr, len, prot);
    } else {
        // Private mapping (existing implementation)
        return mmap_private(addr, len, prot, flags, fd, offset);
    }
}

fn mmap_shared_file(
    addr: usize,
    len: usize,
    prot: u32,
    fd: i32,
    offset: usize,
) -> SyscallResult {
    let file = get_vfs_file(fd as usize)?;

    // Get or create shared page cache for this file
    let pages = file.get_shared_pages(offset, len)?;

    // Map pages into process address space
    let task = los_sched::current_task();
    let vma = task.vmm.lock().map_shared(
        addr,
        len,
        prot,
        pages,
    )?;

    Ok(vma.start as i64)
}
```

**Key change:** Shared mappings use the same physical pages across processes instead of copy-on-write.

**Acceptance:** Two processes can mmap same file with MAP_SHARED and see each other's writes.

---

### UoW 4.2: Shared Page Tracking
**File:** `crates/kernel/vfs/src/file.rs` or new `crates/kernel/mm/src/shared.rs`

Track shared pages for a file:

```rust
/// Shared memory region backed by a file
pub struct SharedMapping {
    /// Physical pages backing this mapping
    pages: Vec<PhysFrame>,
    /// Reference count (how many processes have this mapped)
    refcount: AtomicUsize,
    /// File this is associated with (if any)
    file: Option<Arc<dyn File>>,
}

impl File for MemfdFile {
    fn get_shared_pages(&self, offset: usize, len: usize) -> Result<Arc<SharedMapping>, u32> {
        // Return existing shared mapping or create new one
        let mut shared = self.shared_mapping.lock();
        if shared.is_none() {
            let num_pages = (len + PAGE_SIZE - 1) / PAGE_SIZE;
            let pages = allocate_pages(num_pages)?;
            *shared = Some(Arc::new(SharedMapping {
                pages,
                refcount: AtomicUsize::new(1),
                file: None,
            }));
        }
        Ok(shared.as_ref().unwrap().clone())
    }
}
```

**Acceptance:** Shared pages are reused across mappings.

---

### UoW 4.3: memfd_create() Syscall
**File:** `crates/kernel/syscall/src/mm.rs`

Create anonymous file for shared memory:

```rust
// memfd flags
pub const MFD_CLOEXEC: u32 = 0x0001;
pub const MFD_ALLOW_SEALING: u32 = 0x0002;

/// Create anonymous memory file
///
/// # Arguments
/// * `name` - Name for debugging (shown in /proc/pid/fd/)
/// * `flags` - MFD_CLOEXEC, MFD_ALLOW_SEALING
pub fn sys_memfd_create(name: usize, flags: u32) -> SyscallResult {
    let task = los_sched::current_task();

    // Read name from user space (optional, for debugging)
    let mut name_buf = [0u8; 256];
    let name_str = if name != 0 {
        read_user_cstring(task.ttbr0, name, &mut name_buf).unwrap_or("memfd")
    } else {
        "memfd"
    };

    // Create anonymous file
    let memfd = Arc::new(MemfdFile::new(name_str));

    // Add to fd table
    let cloexec = flags & MFD_CLOEXEC != 0;
    let fd = add_to_fd_table(memfd, cloexec)?;

    Ok(fd as i64)
}
```

**Acceptance:** memfd_create() returns fd that can be mmap'd with MAP_SHARED.

---

### UoW 4.4: MemfdFile Implementation
**File:** `crates/kernel/vfs/src/memfd.rs`

```rust
/// Anonymous memory file for shared memory
pub struct MemfdFile {
    /// Debug name
    name: String,
    /// Current size (set via ftruncate)
    size: AtomicUsize,
    /// Backing pages
    pages: SpinLock<Vec<PhysFrame>>,
    /// Shared mapping for mmap
    shared_mapping: SpinLock<Option<Arc<SharedMapping>>>,
    /// Seals (if MFD_ALLOW_SEALING)
    seals: AtomicU32,
}

impl MemfdFile {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            size: AtomicUsize::new(0),
            pages: SpinLock::new(Vec::new()),
            shared_mapping: SpinLock::new(None),
            seals: AtomicU32::new(0),
        }
    }
}

impl File for MemfdFile {
    fn read(&self, buf: &mut [u8]) -> Result<usize, u32> {
        // Read from backing pages
        todo!()
    }

    fn write(&self, buf: &[u8]) -> Result<usize, u32> {
        // Write to backing pages, expanding as needed
        todo!()
    }

    fn ftruncate(&self, size: i64) -> Result<(), u32> {
        // Resize backing pages
        let size = size as usize;
        let num_pages = (size + PAGE_SIZE - 1) / PAGE_SIZE;

        let mut pages = self.pages.lock();
        while pages.len() < num_pages {
            let frame = allocate_page()?;
            // Zero the page
            unsafe {
                core::ptr::write_bytes(frame.as_virt_ptr::<u8>(), 0, PAGE_SIZE);
            }
            pages.push(frame);
        }
        // Note: shrinking could free pages, but most uses only grow

        self.size.store(size, Ordering::SeqCst);
        Ok(())
    }

    fn get_shared_pages(&self, offset: usize, len: usize) -> Result<Arc<SharedMapping>, u32> {
        // Return pages for mmap(MAP_SHARED)
        let pages = self.pages.lock();
        Ok(Arc::new(SharedMapping {
            pages: pages.clone(),
            refcount: AtomicUsize::new(1),
            file: None,
        }))
    }
}
```

**Acceptance:** memfd can be truncated, mmap'd, and shares memory between processes.

---

### UoW 4.5: Syscall Numbers
**Files:** `crates/kernel/arch/{aarch64,x86_64}/src/lib.rs`

| Syscall | x86_64 | aarch64 |
|---------|--------|---------|
| memfd_create | 319 | 279 |

**Acceptance:** Syscall numbers added and dispatch working.

---

### UoW 4.6: ftruncate for memfd
**File:** `crates/kernel/syscall/src/fs/fd.rs`

Ensure ftruncate works on memfd (it should, via File trait).

**Acceptance:** `ftruncate(memfd, size)` sets size and allocates backing pages.

---

## Testing

### Test 1: Basic memfd + mmap
```c
int fd = memfd_create("test", 0);
ftruncate(fd, 4096);

void *ptr = mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
strcpy(ptr, "hello");

// Re-map and verify
void *ptr2 = mmap(NULL, 4096, PROT_READ, MAP_SHARED, fd, 0);
assert(strcmp(ptr2, "hello") == 0);
```

### Test 2: Cross-process sharing (via fd passing)
```c
// Process A
int fd = memfd_create("shared", 0);
ftruncate(fd, 4096);
void *ptr = mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
strcpy(ptr, "from A");

// Send fd to Process B via unix socket...

// Process B (after receiving fd)
void *ptr = mmap(NULL, 4096, PROT_READ | PROT_WRITE, MAP_SHARED, received_fd, 0);
assert(strcmp(ptr, "from A") == 0);
ptr[0] = 'X';  // Modify

// Back in Process A
assert(ptr[0] == 'X');  // Sees B's modification!
```

### Test 3: Wayland-like pattern
```c
// This is essentially how Wayland wl_shm works:

// Client creates shared buffer
int fd = memfd_create("wl_shm", MFD_CLOEXEC);
ftruncate(fd, width * height * 4);  // ARGB buffer
void *data = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);

// Client draws to buffer...
draw_something(data);

// Client sends fd to compositor via wl_shm protocol
// (uses SCM_RIGHTS over unix socket)

// Compositor mmaps same fd
void *compositor_view = mmap(NULL, size, PROT_READ, MAP_SHARED, client_fd, 0);
// Compositor can now read client's pixels directly!
```

---

## Verification Checklist

- [ ] memfd_create() returns valid fd
- [ ] ftruncate() on memfd allocates pages
- [ ] mmap(MAP_SHARED) on memfd works
- [ ] Two mappings of same memfd share memory
- [ ] fd passing + MAP_SHARED allows cross-process sharing
- [ ] Builds on both x86_64 and aarch64
