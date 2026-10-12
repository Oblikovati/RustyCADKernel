//! Construction policies are private so native handled allocators never recursively construct handles.
use super::{AccEngine, AllocatorKind, BaseAllocator, IncEngine, Mutex};
use crate::standard::Failure;
use std::ptr::NonNull;
impl BaseAllocator {
    pub(in crate::ncollection) fn allocate_calls_optimal(&self) -> bool {
        // Pinned IncAllocator::Allocate forwards virtually to AllocateOptimal.
        matches!(&*self.fields.inner, AllocatorKind::Incremental(_))
    }
    pub(in crate::ncollection) fn optimal_calls_allocate(&self) -> bool {
        // Heap/Acc/Aligned inline bodies call virtual Allocate, unlike Base/Inc bodies.
        matches!(
            &*self.fields.inner,
            AllocatorKind::Heap | AllocatorKind::Accumulating(_) | AllocatorKind::Aligned(_)
        )
    }
    pub(in crate::ncollection) fn common_policy() -> Self {
        let policy = AllocatorKind::Common;
        Self::with_kind(policy)
    }
    pub(in crate::ncollection) fn heap_policy() -> Self {
        let policy = AllocatorKind::Heap;
        Self::with_kind(policy)
    }
    pub(in crate::ncollection) fn incremental_policy(block_size: usize) -> Self {
        let engine = IncEngine::new(block_size);
        Self::with_kind(AllocatorKind::Incremental(Mutex::new(engine)))
    }
    pub(in crate::ncollection) fn accumulating_policy(block_size: usize) -> Result<Self, Failure> {
        let engine = AccEngine::new(block_size)?;
        Ok(Self::with_kind(AllocatorKind::Accumulating(Mutex::new(
            engine,
        ))))
    }
    pub(in crate::ncollection) fn aligned_policy(alignment: usize) -> Self {
        let policy = AllocatorKind::Aligned(alignment);
        Self::with_kind(policy)
    }
    pub(in crate::ncollection) fn allocate_optimal_builtin(
        &self,
        size: usize,
    ) -> Option<NonNull<u8>> {
        let zeroed = !matches!(&*self.fields.inner, AllocatorKind::Common);
        self.allocate_policy(size, zeroed)
    }
}
