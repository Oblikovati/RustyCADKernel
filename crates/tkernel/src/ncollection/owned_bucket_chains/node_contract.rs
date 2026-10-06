//! Prefix casts and payload-derived thread bounds apply only to these audited native node layouts.
use crate::ncollection::{BasicMapNode, DataMapNode, MapNode};
mod sealed {
    pub trait Sealed {}
}
/// # Safety
/// Implementors have a real MapLinkNode prefix at offset zero and exact typed allocation.
/// Key/Value account for every owned payload and threading requirement; links are non-owning.
/// Bucket traversal reads only the link prefix. Key and Value may alias for key-only nodes.
pub(in crate::ncollection) unsafe trait CanonicalMapNode:
    BasicMapNode + sealed::Sealed
{
}
impl<K, V> sealed::Sealed for DataMapNode<K, V> {}
impl<K> sealed::Sealed for MapNode<K> {}
// SAFETY: both repr(C) classes have a real MapLinkNode prefix; their only payloads are K/V.
unsafe impl<K, V> CanonicalMapNode for DataMapNode<K, V> {}
unsafe impl<K> CanonicalMapNode for MapNode<K> {}
