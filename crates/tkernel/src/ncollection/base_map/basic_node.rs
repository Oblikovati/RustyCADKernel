//! Explicit extension contracts replace C++ template member lookup on custom map nodes.
use super::MapLinkNode;

/// Operations used by native `BaseMap::BasicIterator` on a custom node.
/// Example: `impl BasicMapNode for MyNode { /* key, value and link access */ }`.
pub trait BasicMapNode: Sized {
    /// Native node key type, e.g. `i32` for integer map keys.
    type Key;
    /// Native node value type, e.g. `String` for mapped text.
    type Value;
    /// Reads native `Next()` without following it; `node.bucket_next()`.
    fn bucket_next(&self) -> *mut Self;
    /// Reads native `OrderNext()` without following it; `node.order_next()`.
    fn order_next(&self) -> *mut Self;
    /// Borrows native `Key()`; `node.map_key()`.
    fn map_key(&self) -> &Self::Key;
    /// Borrows native const `Value()`; `node.map_value()`.
    fn map_value(&self) -> &Self::Value;
}

/// Writable mapped-value access, omitted for key-only iterator specializations.
/// Example: `impl BasicMapNodeMut for MyNode { fn map_value_mut(&mut self) -> &mut String { &mut self.value } }`.
pub trait BasicMapNodeMut: BasicMapNode {
    /// Borrows native `ChangeValue()` exclusively; `node.map_value_mut()`.
    fn map_value_mut(&mut self) -> &mut Self::Value;
}

/// Supplies the native bucket table and ordered head of a custom container.
/// Example: `impl BasicMapIteratorSource for MyMap { /* node type, buckets and head */ }`.
/// Bucket pointers must address the actual node's `MapLinkNode` prefix for bucket traversal.
pub trait BasicMapIteratorSource {
    /// Actual native node type, e.g. `MyNode` implementing `BasicMapNode`.
    type Node: BasicMapNode;
    /// Borrows all native buckets including bucket zero; `map.basic_bucket_slots()`.
    fn basic_bucket_slots(&self) -> Option<&[*mut MapLinkNode]>;
    /// Returns native `FirstNode()` without traversing it; `map.basic_first_node()`.
    fn basic_first_node(&self) -> *mut Self::Node;
}
