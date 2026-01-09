//! TEAM_202: Dentry (Directory Entry Cache) Implementation
//!
//! The dentry cache (dcache) caches path→inode lookups to avoid
//! repeated filesystem traversals.

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::sync::{Arc, Weak};

use los_utils::RwLock;

use super::error::{VfsError, VfsResult};
use super::inode::Inode;
use super::superblock::Superblock;

/// TEAM_202: Reference to a dentry
pub type DentryRef = Arc<Dentry>;

/// TEAM_202: Weak reference to a dentry
pub type WeakDentryRef = Weak<Dentry>;

/// TEAM_202: Directory Entry Cache Entry
///
/// A dentry caches the mapping from a name to an inode within a directory.
/// Dentries form a tree structure that mirrors the directory hierarchy.
pub struct Dentry {
    /// Name of this entry (e.g., "foo" in "/bar/foo")
    pub name: RwLock<String>,
    /// Parent dentry (None for root)
    pub parent: RwLock<Option<WeakDentryRef>>,
    /// Inode this dentry points to (None = negative dentry)
    pub inode: RwLock<Option<Arc<Inode>>>,
    /// Child dentries
    pub children: RwLock<BTreeMap<String, Arc<Dentry>>>,
    /// Mount point - if another filesystem is mounted here
    pub mounted: RwLock<Option<Arc<dyn Superblock>>>,
}

impl Dentry {
    /// TEAM_202: Create a new dentry
    pub fn new(name: String, parent: Option<WeakDentryRef>, inode: Option<Arc<Inode>>) -> Self {
        Self {
            name: RwLock::new(name),
            parent: RwLock::new(parent),
            inode: RwLock::new(inode),
            children: RwLock::new(BTreeMap::new()),
            mounted: RwLock::new(None),
        }
    }

    /// TEAM_202: Create a root dentry
    pub fn root(inode: Arc<Inode>) -> Arc<Self> {
        Arc::new(Self::new(String::from("/"), None, Some(inode)))
    }

    /// TEAM_202: Check if this is a negative dentry (no inode)
    pub fn is_negative(&self) -> bool {
        self.inode.read().is_none()
    }

    /// TEAM_202: Check if this dentry has a mount
    pub fn is_mountpoint(&self) -> bool {
        self.mounted.read().is_some()
    }

    /// TEAM_202: Get the inode, following mount points
    pub fn get_inode(&self) -> Option<Arc<Inode>> {
        // If something is mounted here, return the root of the mounted fs
        if let Some(ref sb) = *self.mounted.read() {
            return Some(sb.root());
        }
        self.inode.read().clone()
    }

    /// TEAM_202: Look up a child by name
    pub fn lookup_child(&self, name: &str) -> Option<Arc<Dentry>> {
        self.children.read().get(name).cloned()
    }

    /// TEAM_202: Add a child dentry
    pub fn add_child(&self, child: Arc<Dentry>) {
        let child_name = child.name.read().clone();
        self.children.write().insert(child_name, child);
    }

    /// TEAM_202: Remove a child dentry
    pub fn remove_child(&self, name: &str) -> Option<Arc<Dentry>> {
        self.children.write().remove(name)
    }

    /// TEAM_202: Get the full path from root
    pub fn path(&self) -> String {
        let mut components = alloc::vec::Vec::new();

        // Walk up to root, collecting names
        // First, handle self
        let my_name = self.name.read().clone();
        if my_name != "/" {
            components.push(my_name);
        }

        // Then walk up parents
        let mut parent_weak = self.parent.read().clone();
        while let Some(ref weak) = parent_weak {
            if let Some(parent) = weak.upgrade() {
                let p_name = parent.name.read().clone();
                if p_name != "/" {
                    components.push(p_name);
                }
                parent_weak = parent.parent.read().clone();
            } else {
                break;
            }
        }

        // Reverse to get root-to-leaf order
        components.reverse();

        if components.is_empty() {
            String::from("/")
        } else {
            let mut path = String::from("/");
            for (i, comp) in components.iter().enumerate() {
                if i > 0 {
                    path.push('/');
                }
                path.push_str(comp);
            }
            path
        }
    }

    /// TEAM_202: Mount a filesystem at this dentry
    pub fn mount(&self, sb: Arc<dyn Superblock>) {
        *self.mounted.write() = Some(sb);
    }

    /// TEAM_202: Unmount the filesystem at this dentry
    pub fn unmount(&self) -> Option<Arc<dyn Superblock>> {
        self.mounted.write().take()
    }

    /// TEAM_202: Set the inode (for negative→positive transition)
    pub fn set_inode(&self, inode: Arc<Inode>) {
        *self.inode.write() = Some(inode);
    }

    /// TEAM_202: Clear the inode (for deletion)
    pub fn clear_inode(&self) {
        *self.inode.write() = None;
    }
}

impl core::fmt::Debug for Dentry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Dentry")
            .field("name", &*self.name.read())
            .field("is_negative", &self.is_negative())
            .field("is_mountpoint", &self.is_mountpoint())
            .finish()
    }
}

// ============================================================================
// Dentry Cache
// ============================================================================

/// TEAM_202: Global dentry cache
pub struct DentryCache {
    /// Root dentry
    root: RwLock<Option<Arc<Dentry>>>,
}

impl DentryCache {
    /// TEAM_202: Create a new dentry cache
    pub const fn new() -> Self {
        Self {
            root: RwLock::new(None),
        }
    }

    /// TEAM_202: Set the root dentry
    pub fn set_root(&self, dentry: Arc<Dentry>) {
        *self.root.write() = Some(dentry);
    }

    /// TEAM_202: Get the root dentry
    pub fn root(&self) -> Option<Arc<Dentry>> {
        self.root.read().clone()
    }

    /// TEAM_202: Look up a path starting from root
    ///
    /// Returns the final dentry and any remaining path that couldn't be resolved.
    pub fn lookup(&self, path: &str) -> VfsResult<Arc<Dentry>> {
        let root = self.root().ok_or(VfsError::NotFound)?;

        if path == "/" || path.is_empty() {
            return Ok(root);
        }

        let path = path.trim_start_matches('/');
        let mut current = root;

        for component in path.split('/') {
            if component.is_empty() || component == "." {
                continue;
            }

            if component == ".." {
                // Go to parent
                let parent_lock = current.parent.read();
                if let Some(ref weak) = *parent_lock {
                    if let Some(p) = weak.upgrade() {
                        drop(parent_lock);
                        current = p;
                        continue;
                    }
                }
                // Already at root or parent gone
                continue;
            }

            // Look up in cache first
            if let Some(child) = current.lookup_child(component) {
                // Handle mount points
                let is_mp = child.mounted.read().is_some();
                if is_mp {
                    // Create/get the root dentry of the mounted filesystem
                    // For now, just continue with the child
                    current = child;
                    continue;
                }
                current = child;
                continue;
            }

            // Not in cache - need to look up in filesystem
            let inode = current.get_inode().ok_or(VfsError::NotFound)?;

            if !inode.is_dir() {
                return Err(VfsError::NotADirectory);
            }

            // Look up in the actual filesystem
            let child_inode = inode.lookup(component)?;

            // Create a new dentry and cache it
            let child_dentry = Arc::new(Dentry::new(
                String::from(component),
                Some(Arc::downgrade(&current)),
                Some(child_inode),
            ));

            current.add_child(child_dentry.clone());
            current = child_dentry;
        }

        Ok(current)
    }

    /// TEAM_202: Look up parent directory and final component name
    pub fn lookup_parent<'a>(&self, path: &'a str) -> VfsResult<(Arc<Dentry>, &'a str)> {
        let path = path.trim_end_matches('/');

        if path.is_empty() || path == "/" {
            return Err(VfsError::InvalidArgument);
        }

        let (parent_path, name) = match path.rfind('/') {
            Some(idx) => {
                let parent = if idx == 0 { "/" } else { &path[..idx] };
                (&path[idx + 1..], parent)
            }
            None => (path, "/"),
        };

        // Swap: name is parent_path, parent is name in the wrong order
        // Actually: parent_path should be the parent directory, name is the filename
        let (parent_path, name) = (name, parent_path);

        let parent_dentry = self.lookup(parent_path)?;
        Ok((parent_dentry, name))
    }

    /// TEAM_202: Invalidate a path in the cache
    pub fn invalidate(&self, path: &str) {
        // Walk to parent and remove the child
        if let Ok((parent, name)) = self.lookup_parent(path) {
            parent.remove_child(name);
        }
    }
}

impl Default for DentryCache {
    fn default() -> Self {
        Self::new()
    }
}

/// TEAM_202: Global dentry cache instance
static DCACHE: DentryCache = DentryCache::new();

/// TEAM_202: Get the global dentry cache
pub fn dcache() -> &'static DentryCache {
    &DCACHE
}
