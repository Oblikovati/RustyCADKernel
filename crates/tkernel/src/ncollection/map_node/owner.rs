//! Native MapNode owns one key and delegates panic-safe allocation release to the shared owner.
use super::{BaseAllocator, MapLinkNode, MapNode};
use crate::ncollection::{AllocatedBox, node_owner::NativeNodeOwner};
use std::ops::{Deref, DerefMut};
/// Allocator-owned MapNode; `let owner=MapNode::allocate_ref(&arena,&7,next);`.
pub struct MapNodeOwner<K>(NativeNodeOwner<MapNode<K>>);
impl<K> MapNodeOwner<K> {
    pub(super) fn from_allocated(owner: AllocatedBox<MapNode<K>>) -> Self {
        Self(NativeNodeOwner::from_allocated(owner))
    }
    pub(super) fn into_parts(self) -> (*mut MapLinkNode, BaseAllocator) {
        let (pointer, allocator) = self.0.into_parts();
        (pointer.cast(), allocator)
    }
}
impl<K> Deref for MapNodeOwner<K> {
    type Target = MapNode<K>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<K> DerefMut for MapNodeOwner<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
