//! Non-owning link shared by custom native-style bucket nodes.

/// The `NCollection_ListNode` link prefix; no node or successor ownership is implied.
/// Example: `let link = MapLinkNode::new(std::ptr::null_mut());`.
#[repr(C)]
#[derive(Debug)]
pub struct MapLinkNode {
    successor: *mut Self,
}

impl MapLinkNode {
    /// Stores the successor without dereferencing it; `MapLinkNode::new(next)`.
    pub fn new(successor: *mut Self) -> Self {
        Self { successor }
    }
    /// Reads the native const `Next()` pointer; `assert!(link.next().is_null())`.
    pub fn next(&self) -> *mut Self {
        self.successor
    }
    /// Borrows native mutable `Next()`; `*link.next_mut() = next`.
    pub fn next_mut(&mut self) -> &mut *mut Self {
        &mut self.successor
    }
}
