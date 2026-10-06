//! One canonical native base; only its mutable lookup cache changes through shared access.
use super::{BaseAllocator, BaseSequence, SeqNode};
use std::cell::UnsafeCell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

struct SequenceSharedState {
    state: UnsafeCell<BaseSequence>,
    cache_lock: Mutex<()>,
}
// SAFETY: topology edits require exclusivity; shared cache writes hold cache_lock. Pointers
// remain opaque here; Sequence's independent item owners retain T's Send/Sync bounds.
unsafe impl Send for SequenceSharedState {}
unsafe impl Sync for SequenceSharedState {}
// Shared operations write only a non-panicking cache pair under the lock. Poisoning
// from a read guard cannot invalidate topology, so shared unwinding remains safe.
impl std::panic::RefUnwindSafe for SequenceSharedState {}
pub(super) struct SequenceTopology {
    shared: Arc<SequenceSharedState>,
}

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
        unsafe { &*self.topology.shared.state.get() }
    }
}
impl SequenceTopology {
    pub(super) fn new(allocator: BaseAllocator) -> Self {
        Self::from_base(BaseSequence::new(Some(allocator)))
    }
    pub(super) fn from_base(base: BaseSequence) -> Self {
        Self {
            shared: Arc::new(SequenceSharedState {
                state: UnsafeCell::new(base),
                cache_lock: Mutex::new(()),
            }),
        }
    }
    pub(super) fn access(&self) -> SequenceBaseAccess<'_> {
        SequenceBaseAccess {
            topology: self,
            _lock: self
                .shared
                .cache_lock
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        }
    }
    pub(super) fn edit<R>(&mut self, edit: impl FnOnce(&mut BaseSequence) -> R) -> R {
        let _guard = self
            .shared
            .cache_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // SAFETY: exclusive owner and lock exclude observers and shared cache updates.
        edit(unsafe { &mut *self.shared.state.get() })
    }
    pub(super) fn edit_pair<R>(
        &mut self,
        other: &mut Self,
        edit: impl FnOnce(&mut BaseSequence, &mut BaseSequence) -> R,
    ) -> R {
        self.edit(|base| other.edit(|donor| edit(base, donor)))
    }
    pub(super) fn observer(&self) -> SequenceObserver {
        SequenceObserver {
            shared: self.shared.clone(),
        }
    }
    pub(super) fn size(&self) -> usize {
        // Shared cache writes never touch structural fields; &self excludes topology changes.
        unsafe { (*self.shared.state.get()).size }
    }
    pub(super) fn endpoints(&self) -> (*mut SeqNode, *mut SeqNode) {
        // Field reads do not create a shared reference spanning concurrently mutable cache fields.
        unsafe {
            (
                (*self.shared.state.get()).first,
                (*self.shared.state.get()).last,
            )
        }
    }
    pub(super) fn allocator(&self) -> &BaseAllocator {
        // SAFETY: shared methods only modify disjoint current/current_index fields.
        unsafe { &(*self.shared.state.get()).allocator }
    }
    pub(super) fn find(&self, index: usize, update_cache: bool) -> *mut SeqNode {
        let _lock = self
            .shared
            .cache_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let base = self.shared.state.get();
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

/// Read-only canonical metadata access, including during destruction.
/// Example: `let observer = sequence.observer(); assert_eq!(observer.snapshot().size, 2)`.
/// Release any SequenceBaseAccess guard before reading this observer on the same thread.
#[derive(Clone)]
pub struct SequenceObserver {
    shared: Arc<SequenceSharedState>,
}
/// One synchronized read of canonical metadata, never used as authoritative container state.
/// Example: `let observed = sequence.observer().snapshot(); assert_eq!(observed.cached_index, 1)`.
/// Node identities may name destroyed nodes during native Clear; they cannot be dereferenced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SequenceObservation {
    /// Native size at this observation.
    pub size: usize,
    /// Opaque native first-node identity.
    pub first_node: Option<usize>,
    /// Opaque native last-node identity.
    pub last_node: Option<usize>,
    /// Opaque cached-node identity, read together with its index.
    pub cached_node: Option<usize>,
    /// Native one-based cache index, zero for null cache.
    pub cached_index: usize,
}
impl SequenceObserver {
    /// Copies metadata under its synchronization lock; `observer.snapshot().size`.
    pub fn snapshot(&self) -> SequenceObservation {
        let _guard = self
            .shared
            .cache_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        // SAFETY: every structural/cache write takes this lock; only identities escape.
        let base = unsafe { &*self.shared.state.get() };
        SequenceObservation {
            size: base.size,
            first_node: super::iteration::pointer_address(base.first),
            last_node: super::iteration::pointer_address(base.last),
            cached_node: super::iteration::pointer_address(base.current),
            cached_index: base.current_index,
        }
    }
}
