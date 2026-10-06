//! The independent second hash chain owns no target and has native typed-pointer identity.
use super::DoubleMapNode;
impl<K, V> DoubleMapNode<K, V> {
    /// Mutates native Key1; restore bucket invariants before any linked-map lookup.
    /// Example: `*node.change_key1() = 2`.
    pub fn change_key1(&mut self) -> &mut K {
        &mut self.key
    }
    /// Reads native Key2, the same object as Value; `assert_eq!(node.key2(), node.value())`.
    pub fn key2(&self) -> &V {
        &self.value
    }
    /// Borrows native Next2 without owning its target; `*node.next2() = std::ptr::null_mut()`.
    pub fn next2(&mut self) -> &mut *mut Self {
        &mut self.second_next
    }
    /// Inspects Next2 with Rust shared access; `assert!(node.next2_value().is_null())`.
    pub fn next2_value(&self) -> *mut Self {
        self.second_next
    }
    pub(in crate::ncollection) unsafe fn set_second_link(node: *mut Self, next: *mut Self) {
        // SAFETY: caller owns the live node's link mutation; write only Next2, preserving key tags.
        unsafe { std::ptr::addr_of_mut!((*node).second_next).write(next) };
    }
}
