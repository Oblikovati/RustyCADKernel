//! Stable shared tree ownership for native BVH handles retained across rebuilds.
use super::{BvhTree, BvhVector};
use rustycad_tkernel::standard::SharedState;
use std::sync::{Arc, RwLockReadGuard, RwLockWriteGuard};

/// One live BVH tree, shared by its set and every retained handle.
/// Example: `let retained = set.bvh_handle()?; let length = retained.read().length();`.
#[derive(Debug)]
pub struct BvhTreeHandle<V: BvhVector> {
    tree: Arc<SharedState<BvhTree<V>>>,
}
impl<V: BvhVector> Default for BvhTreeHandle<V> {
    fn default() -> Self {
        Self {
            tree: Arc::new(SharedState::new(BvhTree::default())),
        }
    }
}
impl<V: BvhVector> Clone for BvhTreeHandle<V> {
    fn clone(&self) -> Self {
        Self {
            tree: self.tree.clone(),
        }
    }
}
impl<V: BvhVector> BvhTreeHandle<V> {
    /// Borrow the current tree; release the guard before mutating its set.
    /// Example: `let length = handle.read().length();`.
    pub fn read(&self) -> RwLockReadGuard<'_, BvhTree<V>> {
        self.tree.read()
    }
    /// Mutate the shared tree, as through a native non-const handle.
    /// Example: `handle.write().clear()`.
    pub fn write(&self) -> RwLockWriteGuard<'_, BvhTree<V>> {
        self.tree.write()
    }
    /// Compare object identity, independent of tree contents; `first.ptr_eq(&second)`.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.tree, &other.tree)
    }
}
