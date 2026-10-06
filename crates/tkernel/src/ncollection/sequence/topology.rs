//! One canonical native base; only its mutable lookup cache changes through shared access.
use super::{BaseAllocator, BaseSequence, SeqNode};
use std::cell::UnsafeCell;
use std::sync::{Mutex, MutexGuard, PoisonError};

pub(super) struct SequenceTopology {
    state: UnsafeCell<BaseSequence>,
    cache_lock: Mutex<()>,
}
// SAFETY: topology edits require exclusivity; shared cache writes hold cache_lock. Pointers
// remain opaque here; Sequence's independent item owners retain T's Send/Sync bounds.
unsafe impl Send for SequenceTopology {}
unsafe impl Sync for SequenceTopology {}
// Shared operations write only a non-panicking cache pair under the lock. Poisoning
// from a read guard cannot invalidate topology, so shared unwinding remains safe.
impl std::panic::RefUnwindSafe for SequenceTopology {}

/// A locked read of the actual native base, including its lookup cache.
/// Example: `let base = sequence.base_access(); assert_eq!(base.size(), 2)`.
/// Drop this guard before calling another sequence method which accesses its cache.
pub struct SequenceBaseAccess<'a> {
    topology: &'a SequenceTopology,
    _lock: MutexGuard<'a, ()>,
}
impl std::ops::Deref for SequenceBaseAccess<'_> {
    type Target = BaseSequence;
    fn deref(&self) -> &BaseSequence {
        // SAFETY: the guard excludes cache writers; structural changes require &mut Sequence.
        unsafe { &*self.topology.state.get() }
    }
}
impl SequenceTopology {
    pub(super) fn new(allocator: BaseAllocator) -> Self {
        Self::from_base(BaseSequence::new(Some(allocator)))
    }
    pub(super) fn from_base(base: BaseSequence) -> Self {
        Self {
            state: UnsafeCell::new(base),
            cache_lock: Mutex::new(()),
        }
    }
    pub(super) fn access(&self) -> SequenceBaseAccess<'_> {
        SequenceBaseAccess {
            topology: self,
            _lock: self
                .cache_lock
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        }
    }
    pub(super) fn exclusive(&mut self) -> &mut BaseSequence {
        self.state.get_mut()
    }
    pub(super) fn size(&self) -> usize {
        // Shared cache writes never touch structural fields; &self excludes topology changes.
        unsafe { (*self.state.get()).size }
    }
    pub(super) fn endpoints(&self) -> (*mut SeqNode, *mut SeqNode) {
        // Field reads do not create a shared reference spanning concurrently mutable cache fields.
        unsafe { ((*self.state.get()).first, (*self.state.get()).last) }
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        // SAFETY: shared methods only modify disjoint current/current_index fields.
        unsafe { &(*self.state.get()).allocator }
    }
    pub(super) fn find(&self, index: usize, update_cache: bool) -> *mut SeqNode {
        let _lock = self
            .cache_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let base = self.state.get();
        // SAFETY: Sequence checks index and owns a consistent live topology.
        let node = unsafe { (*base).find(index) };
        if update_cache {
            // Field writes avoid forming &mut BaseSequence while allocator() is borrowed.
            unsafe {
                std::ptr::addr_of_mut!((*base).current).write(node);
                std::ptr::addr_of_mut!((*base).current_index).write(index);
            }
        }
        node
    }
}
