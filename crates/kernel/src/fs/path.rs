//! TEAM_201: Filesystem Path Abstraction
//!
//! Provides a zero-cost path abstraction modeled after `std::path::Path`
//! and inspired by Theseus OS's path crate.
//!
//! Key features:
//! - `Path` is a transparent wrapper around `str` (zero-cost)
//! - `PathBuf` is an owned path (wraps `String`)
//! - Component iteration with normalization
//! - Parent/filename extraction

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::ops::Deref;

/// TEAM_201: A borrowed path slice
///
/// This is a zero-cost wrapper around `str`, similar to how `std::path::Path`
/// works. Use `PathBuf` for owned paths.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Path {
    inner: str,
}

impl Path {
    /// TEAM_201: Create a Path from a string slice
    ///
    /// This is a zero-cost conversion.
    #[inline]
    pub fn new<S: AsRef<str> + ?Sized>(s: &S) -> &Path {
        // SAFETY: Path has the same layout as str due to #[repr(transparent)]
        unsafe { &*(s.as_ref() as *const str as *const Path) }
    }

    /// TEAM_201: Get the underlying string slice
    #[inline]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// TEAM_201: Check if this is an absolute path (starts with /)
    #[inline]
    pub fn is_absolute(&self) -> bool {
        self.inner.starts_with('/')
    }

    /// TEAM_201: Check if this is a relative path
    #[inline]
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }

    /// TEAM_201: Iterate over path components
    ///
    /// Normalizes the path during iteration:
    /// - Repeated slashes are collapsed
    /// - `.` components are skipped (except leading `./`)
    /// - Trailing slashes are ignored
    pub fn components(&self) -> Components<'_> {
        Components::new(self)
    }

    /// TEAM_201: Get the parent path
    ///
    /// Returns `None` for root or paths without a parent.
    pub fn parent(&self) -> Option<&Path> {
        let mut components = self.components();
        let last = components.next_back();
        
        last.and_then(|c| match c {
            Component::Normal(_) | Component::CurDir | Component::ParentDir => {
                Some(components.as_path())
            }
            Component::RootDir => None,
        })
    }

    /// TEAM_201: Get the final component (filename or directory name)
    ///
    /// Returns `None` for root or paths ending in `..`
    pub fn file_name(&self) -> Option<&str> {
        self.components().next_back().and_then(|c| match c {
            Component::Normal(name) => Some(name),
            _ => None,
        })
    }

    /// TEAM_201: Join this path with another
    ///
    /// If `path` is absolute, it replaces the current path.
    pub fn join<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let mut buf = self.to_path_buf();
        buf.push(path);
        buf
    }

    /// TEAM_201: Convert to an owned PathBuf
    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf {
            inner: String::from(&self.inner),
        }
    }

    /// TEAM_201: Check if path starts with given prefix
    pub fn starts_with<P: AsRef<Path>>(&self, prefix: P) -> bool {
        let prefix = prefix.as_ref();
        let mut self_components = self.components();
        let mut prefix_components = prefix.components();

        loop {
            match (self_components.next(), prefix_components.next()) {
                (Some(a), Some(b)) if a == b => continue,
                (_, None) => return true,
                _ => return false,
            }
        }
    }

    /// TEAM_201: Strip a prefix from this path
    pub fn strip_prefix<P: AsRef<Path>>(&self, prefix: P) -> Option<&Path> {
        let prefix = prefix.as_ref();
        
        if !self.starts_with(prefix) {
            return None;
        }

        let prefix_str = prefix.as_str().trim_end_matches('/');
        let self_str = self.as_str();
        
        if prefix_str.is_empty() {
            return Some(self);
        }

        let remainder = &self_str[prefix_str.len()..];
        let remainder = remainder.trim_start_matches('/');
        
        Some(Path::new(remainder))
    }
}

impl AsRef<Path> for Path {
    #[inline]
    fn as_ref(&self) -> &Path {
        self
    }
}

impl AsRef<str> for Path {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl AsRef<Path> for str {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self)
    }
}

impl AsRef<Path> for String {
    #[inline]
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// ============================================================================
// PathBuf - Owned path
// ============================================================================

/// TEAM_201: An owned, mutable path
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Default)]
pub struct PathBuf {
    inner: String,
}

impl PathBuf {
    /// TEAM_201: Create an empty PathBuf
    pub fn new() -> Self {
        Self {
            inner: String::new(),
        }
    }

    /// TEAM_201: Create a PathBuf from a String
    pub fn from_string(s: String) -> Self {
        Self { inner: s }
    }

    /// TEAM_201: Push a path onto this one
    ///
    /// If `path` is absolute, it replaces the current path.
    pub fn push<P: AsRef<Path>>(&mut self, path: P) {
        let path = path.as_ref();
        if path.is_absolute() {
            self.inner = String::from(path.as_str());
        } else {
            if !self.inner.is_empty() && !self.inner.ends_with('/') {
                self.inner.push('/');
            }
            self.inner.push_str(path.as_str());
        }
    }

    /// TEAM_201: Remove the last component
    ///
    /// Returns false if the path is empty or only contains root.
    pub fn pop(&mut self) -> bool {
        match self.parent().map(|p| p.as_str().len()) {
            Some(len) => {
                self.inner.truncate(len);
                true
            }
            None => false,
        }
    }

    /// TEAM_201: Convert to the underlying String
    pub fn into_string(self) -> String {
        self.inner
    }

    /// TEAM_201: Get a mutable reference to the underlying String
    pub fn as_mut_string(&mut self) -> &mut String {
        &mut self.inner
    }
}

impl Deref for PathBuf {
    type Target = Path;

    #[inline]
    fn deref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl AsRef<Path> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &Path {
        self.deref()
    }
}

impl AsRef<str> for PathBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl From<String> for PathBuf {
    fn from(s: String) -> Self {
        Self { inner: s }
    }
}

impl From<&str> for PathBuf {
    fn from(s: &str) -> Self {
        Self {
            inner: String::from(s),
        }
    }
}

impl From<&Path> for PathBuf {
    fn from(p: &Path) -> Self {
        p.to_path_buf()
    }
}

impl fmt::Display for PathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}

// ============================================================================
// Component - Path component enumeration
// ============================================================================

/// TEAM_201: A single component of a path
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Component<'a> {
    /// The root directory `/`
    RootDir,
    /// Current directory `.`
    CurDir,
    /// Parent directory `..`
    ParentDir,
    /// A normal component (filename or directory name)
    Normal(&'a str),
}

impl<'a> Component<'a> {
    /// TEAM_201: Convert component to a string slice
    pub fn as_str(&self) -> &'a str {
        match self {
            Component::RootDir => "/",
            Component::CurDir => ".",
            Component::ParentDir => "..",
            Component::Normal(s) => s,
        }
    }
}

impl<'a> AsRef<str> for Component<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> AsRef<Path> for Component<'a> {
    fn as_ref(&self) -> &Path {
        Path::new(self.as_str())
    }
}

// ============================================================================
// Components - Iterator over path components
// ============================================================================

/// TEAM_201: Iterator over the components of a path
pub struct Components<'a> {
    /// The remaining path to iterate over
    path: &'a str,
    /// Whether we've emitted RootDir for an absolute path
    has_root: bool,
    /// Current front position
    front: usize,
    /// Current back position
    back: usize,
}

impl<'a> Components<'a> {
    fn new(path: &'a Path) -> Self {
        let path_str = path.as_str();
        let has_root = path_str.starts_with('/');
        let trimmed = path_str.trim_start_matches('/').trim_end_matches('/');
        
        Self {
            path: trimmed,
            has_root,
            front: 0,
            back: trimmed.len(),
        }
    }

    /// TEAM_201: Get the remaining path as a Path
    pub fn as_path(&self) -> &'a Path {
        if self.has_root && self.front == 0 {
            // Reconstruct with leading slash
            let full_path = unsafe {
                // SAFETY: We know the original path had a leading slash
                let ptr = self.path.as_ptr().sub(1);
                let len = self.back - self.front + 1;
                core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len))
            };
            Path::new(full_path)
        } else {
            Path::new(&self.path[self.front..self.back])
        }
    }
}

impl<'a> Iterator for Components<'a> {
    type Item = Component<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // First, emit RootDir for absolute paths
        if self.has_root && self.front == 0 {
            self.has_root = false;
            return Some(Component::RootDir);
        }

        // Find next component
        loop {
            if self.front >= self.back {
                return None;
            }

            let remaining = &self.path[self.front..self.back];
            let end = remaining.find('/').unwrap_or(remaining.len());
            let component = &remaining[..end];
            
            self.front += end;
            // Skip the slash
            if self.front < self.back {
                self.front += 1;
            }

            // Skip empty components (from double slashes)
            if component.is_empty() {
                continue;
            }

            // Parse the component
            return Some(match component {
                "." => Component::CurDir,
                ".." => Component::ParentDir,
                _ => Component::Normal(component),
            });
        }
    }
}

impl<'a> DoubleEndedIterator for Components<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if self.front >= self.back {
                // Check if we still need to emit RootDir
                if self.has_root {
                    self.has_root = false;
                    return Some(Component::RootDir);
                }
                return None;
            }

            let remaining = &self.path[self.front..self.back];
            let start = remaining.rfind('/').map(|i| i + 1).unwrap_or(0);
            let component = &remaining[start..];
            
            self.back = self.front + start;
            // Skip the slash
            if self.back > self.front && self.back > 0 {
                self.back -= 1;
            }

            // Skip empty components
            if component.is_empty() {
                continue;
            }

            return Some(match component {
                "." => Component::CurDir,
                ".." => Component::ParentDir,
                _ => Component::Normal(component),
            });
        }
    }
}

// ============================================================================
// Utility functions
// ============================================================================

/// TEAM_201: Normalize a path by resolving `.` and `..` components
///
/// This does NOT access the filesystem - it's purely string manipulation.
/// Absolute paths stay absolute, relative paths stay relative.
pub fn normalize(path: &Path) -> PathBuf {
    let mut components: Vec<Component> = Vec::new();
    let is_absolute = path.is_absolute();

    for component in path.components() {
        match component {
            Component::RootDir => {
                components.clear();
                components.push(Component::RootDir);
            }
            Component::CurDir => {
                // Skip `.` components
            }
            Component::ParentDir => {
                match components.last() {
                    Some(Component::Normal(_)) => {
                        components.pop();
                    }
                    Some(Component::RootDir) => {
                        // Can't go above root, ignore
                    }
                    Some(Component::ParentDir) | None => {
                        if !is_absolute {
                            components.push(Component::ParentDir);
                        }
                    }
                    Some(Component::CurDir) => {
                        // Shouldn't happen after normalization
                    }
                }
            }
            Component::Normal(name) => {
                components.push(Component::Normal(name));
            }
        }
    }

    // Build the result
    let mut result = PathBuf::new();
    for (i, component) in components.iter().enumerate() {
        match component {
            Component::RootDir => {
                result.as_mut_string().push('/');
            }
            Component::ParentDir => {
                if i > 0 {
                    result.as_mut_string().push('/');
                }
                result.as_mut_string().push_str("..");
            }
            Component::Normal(name) => {
                if i > 0 && !matches!(components.get(i - 1), Some(Component::RootDir)) {
                    result.as_mut_string().push('/');
                }
                result.as_mut_string().push_str(name);
            }
            Component::CurDir => {}
        }
    }

    if result.inner.is_empty() {
        result.as_mut_string().push('.');
    }

    result
}
