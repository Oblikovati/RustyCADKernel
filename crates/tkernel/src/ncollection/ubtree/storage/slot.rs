//! Unique arena projections never own a pair independently from its registry.
#![allow(unsafe_code)]
use crate::ncollection::AllocatedBox;
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
pub(in crate::ncollection::ubtree) enum TreeNodeSlot<N> {
    Single(AllocatedBox<N>),
    Paired {
        pointer: NonNull<N>,
        pair: usize,
        offset: usize,
    },
}
impl<N> TreeNodeSlot<N> {
    pub(super) fn pair_id(&self) -> Option<usize> {
        match self {
            Self::Paired { pair, .. } => Some(*pair),
            Self::Single(_) => None,
        }
    }
}
impl<N> Deref for TreeNodeSlot<N> {
    type Target = N;
    fn deref(&self) -> &N {
        match self {
            Self::Single(owner) => owner,
            // SAFETY: the owning arena keeps this live pair slot allocated for our borrow.
            Self::Paired { pointer, .. } => unsafe { pointer.as_ref() },
        }
    }
}
impl<N> DerefMut for TreeNodeSlot<N> {
    fn deref_mut(&mut self) -> &mut N {
        match self {
            Self::Single(owner) => owner,
            // SAFETY: there is one projection per live slot; disjoint slot borrows cannot overlap.
            Self::Paired { pointer, .. } => unsafe { pointer.as_mut() },
        }
    }
}
pub(super) fn destroy_single<N>(owner: AllocatedBox<N>) -> AllocatedBox<MaybeUninit<N>> {
    let (pointer, allocator) = owner.into_raw_with_allocator();
    // Clear frees its root after child pairs; the shared immediate node releaser cannot defer Free.
    // SAFETY: this guard owns the same layout, but will only free storage after payload destruction.
    let allocation =
        unsafe { AllocatedBox::from_raw_in(pointer.cast::<MaybeUninit<N>>(), allocator) };
    // SAFETY: the unique original initialized payload is destroyed exactly once.
    unsafe { std::ptr::drop_in_place(pointer) };
    allocation
}
