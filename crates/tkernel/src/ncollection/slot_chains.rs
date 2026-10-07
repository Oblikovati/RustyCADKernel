//! Bucket chains whose links are slot indices: the native `NCollection_BaseMap` table and the
//! `Next()` link of every node, without node pointers.
//!
//! A hashed map keeps its entries in a `Vec` of slots. One `SlotChains` holds one bucket table of
//! `NbBuckets + 1` heads and, for every slot, the next slot of its bucket and that bucket. The
//! stored bucket lets a removal repoint the one link that names a moved slot without hashing, so
//! removal never calls user code. DataMap and Map keep one table over their slots, DoubleMap two
//! over one slot vector, and the indexed and ordered maps add their own per-slot tables beside it.
//! Node addresses are not observable through the public API (plan D1 class B5).
mod bucket_view;
mod entry;
mod position;
mod range;
use crate::{ncollection::base_map::table_slot_count, standard::Failure};
pub use bucket_view::{HashBucketChain, HashBucketTable};
pub(super) use entry::{ChainSlot, drop_in_order};
pub(super) use position::ChainPosition;
pub(super) use range::{ChainView, SlotRange, SlotRangeMut};

/// One chain link: the slot of the next node in the same bucket, or `None` at the chain end.
pub(super) type SlotLink = Option<usize>;

/// The chain state of one slot: its successor and the bucket whose chain holds it.
#[derive(Clone, Copy, Debug)]
struct ChainLink {
    next: SlotLink,
    bucket: usize,
}

/// One bucket table and the per-slot links of its chains; no table exists before the first
/// insertion or after a releasing clear, as in native `myData1 == nullptr`.
#[derive(Debug, Default)]
pub(super) struct SlotChains {
    heads: Option<Box<[SlotLink]>>,
    links: Vec<ChainLink>,
}

impl SlotChains {
    /// Native `myData1 != nullptr`.
    pub(super) fn is_allocated(&self) -> bool {
        self.heads.is_some()
    }
    /// Every bucket head, including bucket zero; empty while no table exists.
    pub(super) fn table(&self) -> &[SlotLink] {
        self.heads.as_deref().unwrap_or(&[])
    }
    /// The first slot of `bucket`; callers pass `hash % NbBuckets + 1` of an allocated table.
    pub(super) fn head(&self, bucket: usize) -> SlotLink {
        self.table()[bucket]
    }
    /// The chain successor of a linked slot, native `Next()`.
    pub(super) fn next(&self, slot: usize) -> SlotLink {
        self.links[slot].next
    }
    /// The first slot of `bucket`'s chain, head first, that `accept` matches.
    pub(super) fn find(&self, bucket: usize, mut accept: impl FnMut(usize) -> bool) -> SlotLink {
        let mut link = self.head(bucket);
        while let Some(slot) = link {
            if accept(slot) {
                return Some(slot);
            }
            link = self.next(slot);
        }
        None
    }
    /// Links a new slot (the next free index) at the head of `bucket`; returns that slot.
    pub(super) fn push_front(&mut self, bucket: usize) -> usize {
        let slot = self.links.len();
        self.links.push(ChainLink { next: None, bucket });
        self.relink_front(slot, bucket);
        slot
    }
    /// Prepends a detached slot to `bucket`, as native `Next() = myData1[b]; myData1[b] = node`.
    pub(super) fn relink_front(&mut self, slot: usize, bucket: usize) {
        let next = self.head_mut(bucket).replace(slot);
        self.links[slot] = ChainLink { next, bucket };
    }
    /// Detaches a linked slot from its chain; its index stays valid until `swap_remove`.
    pub(super) fn unlink(&mut self, slot: usize) {
        let ChainLink { next, bucket } = self.links[slot];
        self.set_link(bucket, slot, next);
    }
    /// Drops the links of an unlinked slot. The last slot moves into it, so the one link that
    /// named the last slot is repointed; the caller moves its own per-slot tables the same way.
    pub(super) fn swap_remove(&mut self, slot: usize) {
        self.links.swap_remove(slot);
        let moved_from = self.links.len();
        if slot < moved_from {
            let bucket = self.links[slot].bucket;
            self.set_link(bucket, moved_from, Some(slot));
        }
    }
    /// Every linked slot in native traversal order: bucket `0..=NbBuckets`, then each chain.
    pub(super) fn order(&self) -> Vec<usize> {
        let mut order = Vec::with_capacity(self.links.len());
        for &head in self.table() {
            let mut link = head;
            while let Some(slot) = link {
                order.push(slot);
                link = self.next(slot);
            }
        }
        order
    }
    /// Native `ReSize` relinking: a new table of `bucket_count + 1` heads, each `(slot, bucket)`
    /// route prepended in the given order. The routes are hashed before any link changes.
    pub(super) fn rebuild(&mut self, bucket_count: usize, routes: &[(usize, usize)]) {
        // Pre-existing D3 site: ReSize has no failure channel; an unaddressable table panics
        // with the native Standard_OutOfMemory failure.
        let mut heads = zeroed_heads(bucket_count).unwrap_or_else(|failure| panic!("{failure}"));
        for &(slot, bucket) in routes {
            let next = heads[bucket].replace(slot);
            self.links[slot] = ChainLink { next, bucket };
        }
        // The new table exists before the old one is freed, so their identities differ.
        self.heads = Some(heads);
    }
    /// Native `destroy`: forgets every link; `release` frees the table, otherwise it is emptied.
    pub(super) fn reset(&mut self, release: bool) {
        self.links.clear();
        match (release, self.heads.as_deref_mut()) {
            (true, _) => self.heads = None,
            (false, Some(heads)) => heads.fill(None),
            (false, None) => {}
        }
    }
    fn head_mut(&mut self, bucket: usize) -> &mut SlotLink {
        let heads = self.heads.as_deref_mut().unwrap_or(&mut []);
        &mut heads[bucket]
    }
    // Replaces the link that names `target` in `bucket` (its head or a predecessor's next).
    // Invariant: every linked slot is in the chain of its stored bucket.
    fn set_link(&mut self, bucket: usize, target: usize, value: SlotLink) {
        let mut previous: SlotLink = None;
        let mut link = self.head(bucket);
        while let Some(slot) = link.filter(|&slot| slot != target) {
            previous = Some(slot);
            link = self.next(slot);
        }
        match previous {
            Some(slot) => self.links[slot].next = value,
            None => *self.head_mut(bucket) = value,
        }
    }
}

fn zeroed_heads(bucket_count: usize) -> Result<Box<[SlotLink]>, Failure> {
    let length = table_slot_count(bucket_count, 1)?;
    let mut heads = Vec::new();
    heads.try_reserve_exact(length).map_err(|_| {
        Failure::with_message(
            "Standard_OutOfMemory",
            &format!(
                "BaseMap bucket allocation of {length} heads failed; expected addressable table"
            ),
        )
    })?;
    heads.resize(length, None);
    Ok(heads.into_boxed_slice())
}
