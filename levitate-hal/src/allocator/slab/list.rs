// TEAM_051: Slab Allocator - Intrusive Linked List
// See docs/planning/slab-allocator/phase-2.md for design

use core::ptr::NonNull;

/// Intrusive list node trait.
/// Types stored in SlabList must implement this to provide list pointers.
pub trait ListNode: Sized {
    fn next(&self) -> Option<NonNull<Self>>;
    fn prev(&self) -> Option<NonNull<Self>>;
    fn set_next(&mut self, next: Option<NonNull<Self>>);
    fn set_prev(&mut self, prev: Option<NonNull<Self>>);
}

/// Intrusive doubly-linked list for slab page management.
///
/// # Invariants
/// - All nodes maintain valid prev/next pointers
/// - head.prev is None
/// - Empty list has head = None and count = 0
pub struct SlabList<T: ListNode> {
    head: Option<NonNull<T>>,
    count: usize,
}

impl<T: ListNode> SlabList<T> {
    /// Create an empty list.
    pub const fn new() -> Self {
        Self {
            head: None,
            count: 0,
        }
    }

    /// Insert node at the front of the list. O(1).
    ///
    /// # Safety
    /// - `node` must be a valid mutable reference
    /// - `node` must not already be in this or another list
    pub fn push_front(&mut self, node: &mut T) {
        let new_node = NonNull::from(&mut *node);

        // Update new node's pointers
        node.set_next(self.head);
        node.set_prev(None);

        // Update old head's prev pointer
        if let Some(mut old_head) = self.head {
            unsafe {
                old_head.as_mut().set_prev(Some(new_node));
            }
        }

        // Update list head
        self.head = Some(new_node);
        self.count += 1;
    }

    /// Remove a specific node from the list. O(1).
    ///
    /// # Safety
    /// - `node` must be in this list
    pub fn remove(&mut self, node: &mut T) {
        let prev = node.prev();
        let next = node.next();

        // Update previous node's next pointer (or head)
        match prev {
            Some(mut prev_node) => {
                // SAFETY: prev_node is valid and within this list
                unsafe {
                    prev_node.as_mut().set_next(next);
                }
            }
            None => {
                // Removing head
                self.head = next;
            }
        }

        // Update next node's prev pointer
        if let Some(mut next_node) = next {
            unsafe {
                next_node.as_mut().set_prev(prev);
            }
        }

        // Clear removed node's pointers
        node.set_next(None);
        node.set_prev(None);

        self.count -= 1;
    }

    /// Remove and return the node at the front of the list. O(1).
    pub fn pop_front(&mut self) -> Option<NonNull<T>> {
        let head = self.head?;

        unsafe {
            let head_ref = head.as_ptr().as_mut().unwrap();
            self.remove(head_ref);
        }

        Some(head)
    }

    /// Check if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Get the number of nodes in the list.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Get head of the list without removing it.
    pub(super) fn head(&self) -> Option<NonNull<T>> {
        self.head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test node implementation
    struct TestNode {
        value: u32,
        next: Option<NonNull<TestNode>>,
        prev: Option<NonNull<TestNode>>,
    }

    impl TestNode {
        fn new(value: u32) -> Self {
            Self {
                value,
                next: None,
                prev: None,
            }
        }
    }

    impl ListNode for TestNode {
        fn next(&self) -> Option<NonNull<Self>> {
            self.next
        }
        fn prev(&self) -> Option<NonNull<Self>> {
            self.prev
        }
        fn set_next(&mut self, next: Option<NonNull<Self>>) {
            self.next = next;
        }
        fn set_prev(&mut self, prev: Option<NonNull<Self>>) {
            self.prev = prev;
        }
    }

    #[test]
    fn test_new_list_is_empty() {
        let list: SlabList<TestNode> = SlabList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_push_front_adds_to_head() {
        let mut list = SlabList::new();
        let mut node1 = TestNode::new(1);
        let mut node2 = TestNode::new(2);

        list.push_front(&mut node1);
        assert_eq!(list.len(), 1);
        assert!(!list.is_empty());

        list.push_front(&mut node2);
        assert_eq!(list.len(), 2);

        // Head should be node2
        let head = list.head.unwrap();
        unsafe {
            assert_eq!(head.as_ref().value, 2);
        }
    }

    #[test]
    fn test_pop_front_returns_head() {
        let mut list = SlabList::new();
        let mut node1 = TestNode::new(1);
        let mut node2 = TestNode::new(2);

        list.push_front(&mut node1);
        list.push_front(&mut node2);

        let popped = list.pop_front().unwrap();
        unsafe {
            assert_eq!(popped.as_ref().value, 2);
        }
        assert_eq!(list.len(), 1);

        let popped = list.pop_front().unwrap();
        unsafe {
            assert_eq!(popped.as_ref().value, 1);
        }
        assert_eq!(list.len(), 0);
        assert!(list.is_empty());
    }

    #[test]
    fn test_remove_from_middle() {
        let mut list = SlabList::new();
        let mut node1 = TestNode::new(1);
        let mut node2 = TestNode::new(2);
        let mut node3 = TestNode::new(3);

        list.push_front(&mut node3);
        list.push_front(&mut node2);
        list.push_front(&mut node1);

        // List: 1 -> 2 -> 3
        list.remove(&mut node2);
        assert_eq!(list.len(), 2);

        // Verify 1 -> 3
        let head = list.head.unwrap();
        unsafe {
            assert_eq!(head.as_ref().value, 1);
            let next = head.as_ref().next.unwrap();
            assert_eq!(next.as_ref().value, 3);
        }
    }

    #[test]
    fn test_remove_head() {
        let mut list = SlabList::new();
        let mut node1 = TestNode::new(1);
        let mut node2 = TestNode::new(2);

        list.push_front(&mut node2);
        list.push_front(&mut node1);

        list.remove(&mut node1);
        assert_eq!(list.len(), 1);

        let head = list.head.unwrap();
        unsafe {
            assert_eq!(head.as_ref().value, 2);
        }
    }

    #[test]
    fn test_empty_list_pop() {
        let mut list: SlabList<TestNode> = SlabList::new();
        assert!(list.pop_front().is_none());
    }
}
