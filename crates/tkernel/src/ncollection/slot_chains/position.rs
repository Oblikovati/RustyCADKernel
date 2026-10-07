//! Native iteration follows one chain, then scans forward for the next nonempty bucket.
use super::{SlotChains, SlotLink};

/// A traversal point: the current bucket and the current slot (`None` once exhausted).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ncollection) struct ChainPosition {
    bucket: usize,
    slot: SlotLink,
}

impl ChainPosition {
    /// The first entry in bucket order, or an exhausted position for an empty or unallocated map.
    pub(in crate::ncollection) fn first(chains: &SlotChains) -> Self {
        let mut position = Self::default();
        position.seek(chains);
        position
    }
    /// The current slot; `None` means native `More()` is false.
    pub(in crate::ncollection) fn slot(&self) -> SlotLink {
        self.slot
    }
    /// Native `Next()`: the chain successor, else the head of the next nonempty bucket.
    pub(in crate::ncollection) fn advance(&mut self, chains: &SlotChains) {
        let Some(slot) = self.slot else {
            return;
        };
        self.slot = chains.next(slot);
        if self.slot.is_some() {
            return;
        }
        self.bucket += 1;
        self.seek(chains);
    }
    fn seek(&mut self, chains: &SlotChains) {
        let table = chains.table();
        while let Some(&head) = table.get(self.bucket) {
            if head.is_some() {
                self.slot = head;
                return;
            }
            self.bucket += 1;
        }
        self.slot = None;
    }
}
