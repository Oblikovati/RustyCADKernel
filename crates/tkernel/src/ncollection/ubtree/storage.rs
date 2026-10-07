//! Canonical node slots and real two-node allocations retain independent lifetimes.
#![allow(unsafe_code)]
use crate::ncollection::{AllocatedBox, BaseAllocator};
use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
    ptr::NonNull,
};
mod pair;
mod slot;
use pair::TreePairAllocation;
use slot::{TreeNodeSlot, destroy_single};
pub(super) struct TreeArena<N> {
    slots: Vec<Option<TreeNodeSlot<N>>>,
    pairs: Vec<Option<TreePairAllocation<N>>>,
}
// SAFETY: the complete private arena owns every projection and pair; none escape independently.
// Mutations require &mut and shared queries access only N. Allocator handles are thread-safe.
unsafe impl<N: Send> Send for TreeArena<N> {}
unsafe impl<N: Sync> Sync for TreeArena<N> {}
impl<N> TreeArena<N> {
    pub(super) fn new() -> Self {
        Self {
            slots: Vec::new(),
            pairs: Vec::new(),
        }
    }
    pub(super) fn len(&self) -> usize {
        self.slots.len()
    }
    pub(super) fn get(&self, index: usize) -> Option<&Option<TreeNodeSlot<N>>> {
        self.slots.get(index)
    }
    pub(super) fn get_mut(&mut self, index: usize) -> Option<&mut Option<TreeNodeSlot<N>>> {
        self.slots.get_mut(index)
    }
    pub(super) fn push_single(&mut self, owner: AllocatedBox<N>) -> usize {
        let index = self.slots.len();
        self.slots.push(Some(TreeNodeSlot::Single(owner)));
        index
    }
    pub(super) fn get_disjoint_mut<const COUNT: usize>(
        &mut self,
        indices: [usize; COUNT],
    ) -> Result<[&mut Option<TreeNodeSlot<N>>; COUNT], std::slice::GetDisjointMutError> {
        self.slots.get_disjoint_mut(indices)
    }
    pub(super) fn insert_pair(&mut self, pair: TreePairAllocation<N>) -> usize {
        self.slots.reserve(2);
        self.pairs.reserve(1);
        let first = self.slots.len();
        let pair_id = self.pairs.len();
        for offset in 0..2 {
            self.slots.push(Some(TreeNodeSlot::Paired {
                pointer: NonNull::new(pair.pointer(offset)).unwrap(),
                pair: pair_id,
                offset,
            }));
        }
        self.pairs.push(Some(pair));
        first
    }
    pub(super) fn allocate_pair(
        allocator: &BaseAllocator,
        construct: impl FnMut(usize) -> N,
    ) -> TreePairAllocation<N> {
        TreePairAllocation::new_with(allocator, construct)
    }
    pub(super) fn child_pairs(&self, children: [usize; 2]) -> Vec<usize> {
        let mut pairs = Vec::new();
        self.record_child_pairs(children, &mut pairs);
        pairs
    }
    pub(super) fn record_child_pairs(&self, children: [usize; 2], pairs: &mut Vec<usize>) {
        let first = self.pair_id(children[0]);
        let second = self.pair_id(children[1]);
        pairs.extend(first);
        if second != first {
            pairs.extend(second);
        }
    }
    fn pair_id(&self, index: usize) -> Option<usize> {
        self.slots.get(index)?.as_ref()?.pair_id()
    }
    pub(super) fn destroy(&mut self, index: usize) -> Option<AllocatedBox<MaybeUninit<N>>> {
        match self.slots[index].take().unwrap() {
            TreeNodeSlot::Single(owner) => Some(destroy_single(owner)),
            TreeNodeSlot::Paired { pair, offset, .. } => {
                self.pairs[pair].as_mut().unwrap().destroy(offset);
                None
            }
        }
    }
    pub(super) fn release_pair(&mut self, index: usize) {
        let Some(pair) = self.pairs[index].as_ref() else {
            return;
        };
        if pair.is_empty() {
            self.pairs[index].take();
        }
    }
    pub(super) fn clear(&mut self) {
        self.slots.clear();
        self.pairs.clear();
    }
}
impl<N> Index<usize> for TreeArena<N> {
    type Output = Option<TreeNodeSlot<N>>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}
impl<N> IndexMut<usize> for TreeArena<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slots[index]
    }
}
