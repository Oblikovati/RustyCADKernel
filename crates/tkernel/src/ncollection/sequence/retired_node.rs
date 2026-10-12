//! Retire the real detached allocation after releasing a collection adapter's field guard.
use super::{SequenceNodeOwner, owner};

/// Unique owner of an unlinked native node; `drop(sequence.detach_cursor(&mut cursor)?)`.
pub struct SequenceNodeRetirement<T> {
    pub(super) owner: SequenceNodeOwner<T>,
}
impl<T> SequenceNodeRetirement<T> {
    /// Borrow the actual detached payload before destruction; `retired.value()`.
    pub fn value(&self) -> &T {
        // SAFETY: this owner keeps the original allocation alive and excludes mutable access.
        unsafe { owner::item_ref(self.owner.link) }
    }
}

#[cfg(test)]
mod tests {
    use crate::ncollection::{Sequence, SequenceCursor};
    use crate::standard::SharedState;
    use std::sync::{Arc, Mutex, Weak};

    struct DetachedSequenceDropProbe {
        outer_sequence: Weak<SharedState<Sequence<Self>>>,
        lengths_at_drop: Arc<Mutex<Vec<usize>>>,
        invoke_callback: bool,
    }
    impl Drop for DetachedSequenceDropProbe {
        fn drop(&mut self) {
            if !self.invoke_callback {
                return;
            }
            let Some(scope) = self.outer_sequence.upgrade() else {
                return;
            };
            let length = scope.read().len();
            self.lengths_at_drop.lock().unwrap().push(length);
        }
    }
    type DropScope = Arc<SharedState<Sequence<DetachedSequenceDropProbe>>>;
    fn populated_drop_scope() -> (DropScope, Arc<Mutex<Vec<usize>>>) {
        let scope = Arc::new(SharedState::new(Sequence::default()));
        let lengths = Arc::new(Mutex::new(Vec::new()));
        for invoke_callback in [true, false] {
            scope.write().append(DetachedSequenceDropProbe {
                outer_sequence: Arc::downgrade(&scope),
                lengths_at_drop: lengths.clone(),
                invoke_callback,
            });
        }
        (scope, lengths)
    }
    #[test]
    fn detached_node_reenters_collection_after_native_metadata_and_guard_release() {
        let (scope, lengths) = populated_drop_scope();
        let mut cursor = scope.read().cursor();
        let address = std::ptr::from_ref(cursor.value(&scope.read()).unwrap());
        let retired = scope.write().detach_cursor(&mut cursor).unwrap().unwrap();
        assert_eq!(std::ptr::from_ref(retired.value()), address);
        assert_eq!(scope.read().len(), 1);
        assert!(cursor.more());
        assert!(lengths.lock().unwrap().is_empty());
        drop(retired);
        assert_eq!(*lengths.lock().unwrap(), vec![1]);
    }
    #[test]
    fn detached_end_is_noop_and_foreign_node_is_rejected_before_editing() {
        let mut sequence: Sequence<i32> = [1].into_iter().collect();
        let other: Sequence<i32> = [2].into_iter().collect();
        let mut foreign = other.cursor();
        let failure = sequence.detach_cursor(&mut foreign).err().unwrap();
        assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
        assert_eq!(sequence.len(), 1);
        assert!(
            sequence
                .detach_cursor(&mut SequenceCursor::default())
                .unwrap()
                .is_none()
        );
        assert_eq!(sequence.first(), Some(&1));
    }
}
