//! Stable canonical metadata permits controlled observation between short mutation phases.
use super::{BaseAllocator, BaseListStorage};
use std::{
    cell::UnsafeCell,
    sync::{Arc, Mutex, PoisonError},
};
struct ListSharedState {
    base: UnsafeCell<BaseListStorage>,
    lock: Mutex<()>,
}
// SAFETY: pointers are opaque metadata; writes require the sole owner and the lock.
// Observers never dereference nodes, so item Send/Sync bounds remain on List's owners.
unsafe impl Send for ListSharedState {}
unsafe impl Sync for ListSharedState {}
impl std::panic::RefUnwindSafe for ListSharedState {}
pub(super) struct ListTopology {
    shared: Arc<ListSharedState>,
}
/// Read-only access to an owner's actual native metadata, including during item Drop.
/// Example: `let observer = list.observer(); assert_eq!(observer.size(), list.len())`.
/// Node identities are opaque and may name already destroyed nodes during native Clear.
#[derive(Clone)]
pub struct ListObserver {
    shared: Arc<ListSharedState>,
}
impl ListTopology {
    pub(super) fn new(allocator: BaseAllocator) -> Self {
        Self {
            shared: Arc::new(ListSharedState {
                base: UnsafeCell::new(BaseListStorage::new(Some(allocator))),
                lock: Mutex::new(()),
            }),
        }
    }
    pub(super) fn base(&self) -> &BaseListStorage {
        // SAFETY: an owner borrow excludes structural mutation; observers only read metadata.
        unsafe { &*self.shared.base.get() }
    }
    pub(super) fn edit<R>(&mut self, edit: impl FnOnce(&mut BaseListStorage) -> R) -> R {
        let _guard = self
            .shared
            .lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // SAFETY: the sole owner's exclusive borrow and lock exclude all other metadata access.
        edit(unsafe { &mut *self.shared.base.get() })
    }
    pub(super) fn edit_pair<R>(
        &mut self,
        other: &mut Self,
        edit: impl FnOnce(&mut BaseListStorage, &mut BaseListStorage) -> R,
    ) -> R {
        self.edit(|base| other.edit(|donor| edit(base, donor)))
    }
    pub(super) fn exchange(&mut self, other: &mut Self) {
        self.edit_pair(other, BaseListStorage::exchange);
    }
    pub(super) fn observer(&self) -> ListObserver {
        ListObserver {
            shared: self.shared.clone(),
        }
    }
}
impl ListObserver {
    fn inspect<R>(&self, inspect: impl FnOnce(&BaseListStorage) -> R) -> R {
        let _guard = self
            .shared
            .lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // SAFETY: the lock excludes every structural write; inspect cannot retain this borrow.
        inspect(unsafe { &*self.shared.base.get() })
    }
    /// Reads native Size, including its intermediate destructor value; `observer.size()`.
    pub fn size(&self) -> usize {
        self.inspect(BaseListStorage::size)
    }
    /// Reads native head-pointer emptiness independently of Size; `observer.is_empty()`.
    pub fn is_empty(&self) -> bool {
        self.inspect(BaseListStorage::is_empty)
    }
    /// Reads opaque head identity without dereferencing it; `observer.first_node()`.
    pub fn first_node(&self) -> Option<usize> {
        self.inspect(|base| super::address(base.first))
    }
    /// Reads opaque tail identity without dereferencing it; `observer.last_node()`.
    pub fn last_node(&self) -> Option<usize> {
        self.inspect(|base| super::address(base.last))
    }
}
