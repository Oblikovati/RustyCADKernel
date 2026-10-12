//! Allocate and construct before taking the collection's edit guard, retaining the real node.
use super::{BaseAllocator, Sequence, SequenceNodeOwner, owner};

/// One allocated but unlinked native node; `let node=SequencePreparedNode::new(&arena,||value)`.
pub struct SequencePreparedNode<T> {
    owner: SequenceNodeOwner<T>,
}
impl<T> SequencePreparedNode<T> {
    /// Allocate before invoking the constructor; `SequencePreparedNode::new(&arena,||value)`.
    pub fn new(allocator: &BaseAllocator, construct: impl FnOnce() -> T) -> Self {
        Self {
            owner: SequenceNodeOwner::construct(allocator, construct),
        }
    }
    /// Borrow the actual unlinked payload; `assert_eq!(prepared.value(),&7)`.
    pub fn value(&self) -> &T {
        // SAFETY: this owner keeps its typed allocation alive and excludes mutable access.
        unsafe { owner::item_ref(self.owner.link) }
    }
}
impl<T> Sequence<T> {
    /// Append an already constructed native node at the current tail; `sequence.append_prepared(node)`.
    pub fn append_prepared(&mut self, node: SequencePreparedNode<T>) -> &mut T {
        let link = node.owner.link;
        self.owners.insert(link as usize, node.owner);
        // Allocation callbacks may have edited the list: native PAppend uses the current tail.
        self.topology
            .edit(|base| unsafe { base.insert_after(base.size(), link) })
            .unwrap_or_else(|failure| {
                panic!(
                    "prepared node {link:p}; expected current-tail append: {}",
                    failure.message()
                )
            });
        // SAFETY: the registry uniquely owns the inserted node under this exclusive borrow.
        unsafe { owner::item_mut(link) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct PreparedDropCounter(Arc<AtomicUsize>);
    impl Drop for PreparedDropCounter {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[test]
    fn prepared_node_keeps_physical_payload_identity_across_intervening_edits() {
        let mut sequence = Sequence::default();
        let node = SequencePreparedNode::new(&BaseAllocator::common(), || 7);
        let address = std::ptr::from_ref(node.value());
        sequence.append(3);
        let inserted = sequence.append_prepared(node);
        assert_eq!(std::ptr::from_ref(inserted), address);
        assert_eq!(sequence.iter().copied().collect::<Vec<_>>(), vec![3, 7]);
    }
    #[test]
    fn prepared_node_releases_exactly_one_payload_when_unlinked_or_inserted() {
        let drops = Arc::new(AtomicUsize::new(0));
        let arena = BaseAllocator::common();
        let node = SequencePreparedNode::new(&arena, || PreparedDropCounter(drops.clone()));
        drop(node);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let mut sequence = Sequence::default();
        sequence.append_prepared(SequencePreparedNode::new(&arena, || {
            PreparedDropCounter(drops.clone())
        }));
        sequence.clear(None);
        assert_eq!(drops.load(Ordering::SeqCst), 2);
    }
}
