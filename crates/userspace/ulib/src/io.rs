//! TEAM_168: I/O abstractions for LevitateOS userspace.
//!
//! Provides error types, traits, and common I/O functionality.
//! TEAM_180: Added BufReader and BufWriter for buffered I/O.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// TEAM_168: Error codes from syscalls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// Function not implemented (ENOSYS = -1)
    NotImplemented,
    /// Bad file descriptor (EBADF = -2)
    BadFd,
    /// Bad address (EFAULT = -3)
    BadAddress,
    /// Invalid argument (EINVAL = -4)
    InvalidArgument,
    /// No such file or directory (ENOENT = -5)
    NotFound,
    /// Too many open files (EMFILE = -6)
    TooManyFiles,
    /// TEAM_183: Not a directory (ENOTDIR = -7)
    NotADirectory,
    /// TEAM_182: Unexpected end of file (internal, not from syscall)
    UnexpectedEof,
    /// TEAM_182: Write returned zero bytes (internal, not from syscall)
    WriteZero,
    /// Unknown error
    Unknown,
}

impl ErrorKind {
    /// TEAM_168: Convert from syscall return value.
    /// TEAM_183: Added ENOTDIR mapping.
    pub fn from_errno(errno: isize) -> Self {
        match errno {
            -1 => Self::NotImplemented,
            -2 => Self::BadFd,
            -3 => Self::BadAddress,
            -4 => Self::InvalidArgument,
            -5 => Self::NotFound,
            -6 => Self::TooManyFiles,
            -7 => Self::NotADirectory,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "function not implemented"),
            Self::BadFd => write!(f, "bad file descriptor"),
            Self::BadAddress => write!(f, "bad address"),
            Self::InvalidArgument => write!(f, "invalid argument"),
            Self::NotFound => write!(f, "no such file or directory"),
            Self::TooManyFiles => write!(f, "too many open files"),
            Self::NotADirectory => write!(f, "not a directory"),
            Self::UnexpectedEof => write!(f, "unexpected end of file"),
            Self::WriteZero => write!(f, "write returned zero bytes"),
            Self::Unknown => write!(f, "unknown error"),
        }
    }
}

/// TEAM_168: I/O Error type.
/// TEAM_187: Added PartialEq, Eq for error comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// TEAM_168: Create a new error from an error kind.
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// TEAM_168: Create an error from a syscall return value.
    pub fn from_errno(errno: isize) -> Self {
        Self::new(ErrorKind::from_errno(errno))
    }

    /// TEAM_168: Get the error kind.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

/// TEAM_187: Implement core::error::Error for consistency with kernel errors.
impl core::error::Error for Error {}

/// TEAM_168: Result type for I/O operations.
pub type Result<T> = core::result::Result<T, Error>;

/// TEAM_168: Read trait for types that can be read from.
pub trait Read {
    /// Read bytes into a buffer.
    ///
    /// Returns the number of bytes read, or an error.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    /// Read exactly `buf.len()` bytes.
    ///
    /// TEAM_182: Returns UnexpectedEof if EOF is reached before filling buffer.
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.read(&mut buf[offset..])?;
            if n == 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof));
            }
            offset += n;
        }
        Ok(())
    }
}

/// TEAM_168: Write trait for types that can be written to.
pub trait Write {
    /// Write bytes from a buffer.
    ///
    /// Returns the number of bytes written, or an error.
    fn write(&mut self, buf: &[u8]) -> Result<usize>;

    /// Flush any buffered data.
    fn flush(&mut self) -> Result<()>;

    /// Write all bytes from a buffer.
    ///
    /// TEAM_182: Returns WriteZero if write returns 0 before all bytes written.
    fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        let mut offset = 0;
        while offset < buf.len() {
            let n = self.write(&buf[offset..])?;
            if n == 0 {
                return Err(Error::new(ErrorKind::WriteZero));
            }
            offset += n;
        }
        Ok(())
    }
}

// ============================================================================
// Buffered I/O (TEAM_180)
// ============================================================================

/// TEAM_180: Default buffer size for BufReader and BufWriter (Q1 decision: 8 KB).
pub const DEFAULT_BUF_CAPACITY: usize = 8192;

/// TEAM_180: Buffered reader wrapper.
///
/// Wraps any `Read` implementor with an internal buffer to reduce syscall overhead.
///
/// # Example
/// ```rust
/// use ulib::fs::File;
/// use ulib::io::{BufReader, Read};
///
/// let file = File::open("/config.txt")?;
/// let mut reader = BufReader::new(file);
/// let mut line = String::new();
/// reader.read_line(&mut line)?;
/// ```
pub struct BufReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,  // Next byte to read from buffer
    cap: usize,  // Valid bytes in buffer (buf[0..cap] is valid data)
}

impl<R: Read> BufReader<R> {
    /// TEAM_180: Create a new BufReader with default buffer capacity.
    pub fn new(inner: R) -> Self {
        Self::with_capacity(DEFAULT_BUF_CAPACITY, inner)
    }

    /// TEAM_180: Create a new BufReader with custom buffer capacity.
    ///
    /// TEAM_182: Capacity of 0 is treated as DEFAULT_BUF_CAPACITY to avoid edge cases.
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        let actual_capacity = if capacity == 0 { DEFAULT_BUF_CAPACITY } else { capacity };
        let mut buf = Vec::with_capacity(actual_capacity);
        buf.resize(actual_capacity, 0);
        Self {
            inner,
            buf,
            pos: 0,
            cap: 0,
        }
    }

    /// TEAM_180: Get reference to underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// TEAM_180: Get mutable reference to underlying reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// TEAM_180: Consume and return underlying reader.
    pub fn into_inner(self) -> R {
        self.inner
    }

    /// TEAM_180: Returns currently buffered data without consuming.
    pub fn buffer(&self) -> &[u8] {
        &self.buf[self.pos..self.cap]
    }

    /// TEAM_180: Fill the internal buffer from the underlying reader.
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.pos >= self.cap {
            // Buffer exhausted, refill
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    /// TEAM_180: Consume n bytes from the buffer.
    /// TEAM_184: Use saturating_add to prevent overflow.
    fn consume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_add(amt).min(self.cap);
    }

    /// TEAM_180: Read a line into the provided String (Q5: includes newline, Q7: appends).
    ///
    /// Returns bytes read (including newline), or 0 at EOF.
    /// Q6: For binary files without newlines, reads up to buffer size.
    pub fn read_line(&mut self, buf: &mut String) -> Result<usize> {
        let mut total_read = 0;

        loop {
            let available = self.fill_buf()?;
            if available.is_empty() {
                // EOF reached
                return Ok(total_read);
            }

            // Look for newline in available data
            let newline_pos = available.iter().position(|&b| b == b'\n');

            match newline_pos {
                Some(pos) => {
                    // Found newline - read up to and including it (Q5)
                    let to_read = pos + 1;
                    let slice = &available[..to_read];
                    
                    // Convert to string and append (Q7: don't clear)
                    match core::str::from_utf8(slice) {
                        Ok(s) => buf.push_str(s),
                        Err(_) => return Err(Error::new(ErrorKind::InvalidArgument)),
                    }
                    
                    self.consume(to_read);
                    total_read += to_read;
                    return Ok(total_read);
                }
                None => {
                    // No newline found - read all available (Q6: up to buffer size)
                    let to_read = available.len();
                    
                    match core::str::from_utf8(available) {
                        Ok(s) => buf.push_str(s),
                        Err(_) => return Err(Error::new(ErrorKind::InvalidArgument)),
                    }
                    
                    self.consume(to_read);
                    total_read += to_read;
                    
                    // Q6: If we've read a full buffer without newline, return
                    // to avoid infinite loop on binary files
                    if to_read >= self.buf.len() {
                        return Ok(total_read);
                    }
                    // Otherwise continue looking for newline
                }
            }
        }
    }
}

impl<R: Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // TEAM_184: Handle empty buffer without syscall (per Read trait contract)
        if buf.is_empty() {
            return Ok(0);
        }

        // Q2: Return buffered data immediately if available
        let available = self.fill_buf()?;
        if available.is_empty() {
            return Ok(0);
        }

        let to_copy = buf.len().min(available.len());
        buf[..to_copy].copy_from_slice(&available[..to_copy]);
        self.consume(to_copy);
        Ok(to_copy)
    }
}

/// TEAM_180: Buffered writer wrapper.
///
/// Wraps any `Write` implementor with an internal buffer to reduce syscall overhead.
/// Automatically flushes when buffer is full (Q3) and on drop (Q4: best-effort).
///
/// # Example
/// ```rust
/// use ulib::io::{BufWriter, Write};
///
/// let mut writer = BufWriter::new(stdout);
/// writer.write_all(b"Hello, buffered world!\n")?;
/// // Auto-flushed when writer drops
/// ```
pub struct BufWriter<W: Write> {
    inner: Option<W>,  // Option for take() in drop
    buf: Vec<u8>,
}

impl<W: Write> BufWriter<W> {
    /// TEAM_180: Create a new BufWriter with default buffer capacity.
    pub fn new(inner: W) -> Self {
        Self::with_capacity(DEFAULT_BUF_CAPACITY, inner)
    }

    /// TEAM_180: Create a new BufWriter with custom buffer capacity.
    ///
    /// TEAM_182: Capacity of 0 is treated as DEFAULT_BUF_CAPACITY to avoid edge cases.
    pub fn with_capacity(capacity: usize, inner: W) -> Self {
        let actual_capacity = if capacity == 0 { DEFAULT_BUF_CAPACITY } else { capacity };
        Self {
            inner: Some(inner),
            buf: Vec::with_capacity(actual_capacity),
        }
    }

    /// TEAM_180: Get reference to underlying writer.
    pub fn get_ref(&self) -> Option<&W> {
        self.inner.as_ref()
    }

    /// TEAM_180: Get mutable reference to underlying writer.
    pub fn get_mut(&mut self) -> Option<&mut W> {
        self.inner.as_mut()
    }

    /// TEAM_180: Consume, flush, and return underlying writer.
    pub fn into_inner(mut self) -> Result<W> {
        self.flush_buf()?;
        Ok(self.inner.take().unwrap())
    }

    /// TEAM_180: Returns buffered data waiting to be written.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }

    /// TEAM_180: Flush internal buffer to underlying writer.
    fn flush_buf(&mut self) -> Result<()> {
        if self.buf.is_empty() {
            return Ok(());
        }

        let inner = match self.inner.as_mut() {
            Some(w) => w,
            None => return Err(Error::new(ErrorKind::BadFd)),
        };

        let mut written = 0;
        while written < self.buf.len() {
            let n = inner.write(&self.buf[written..])?;
            if n == 0 {
                // TEAM_184: Use WriteZero instead of Unknown for consistency
                return Err(Error::new(ErrorKind::WriteZero));
            }
            written += n;
        }
        self.buf.clear();
        Ok(())
    }
}

impl<W: Write> Write for BufWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let capacity = self.buf.capacity();
        
        // Q3: Flush when buffer would overflow
        if self.buf.len() + buf.len() > capacity {
            self.flush_buf()?;
        }

        // If input is larger than buffer capacity, write directly
        if buf.len() >= capacity {
            let inner = match self.inner.as_mut() {
                Some(w) => w,
                None => return Err(Error::new(ErrorKind::BadFd)),
            };
            return inner.write(buf);
        }

        // Q8: Buffer the data, return bytes actually buffered
        let space_available = capacity - self.buf.len();
        let to_buffer = buf.len().min(space_available);
        self.buf.extend_from_slice(&buf[..to_buffer]);
        Ok(to_buffer)
    }

    fn flush(&mut self) -> Result<()> {
        self.flush_buf()?;
        if let Some(inner) = self.inner.as_mut() {
            inner.flush()?;
        }
        Ok(())
    }
}

impl<W: Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        // Q4: Best-effort flush, ignore errors (can't propagate from Drop)
        let _ = self.flush_buf();
    }
}
