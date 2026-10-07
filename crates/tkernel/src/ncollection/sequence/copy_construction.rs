//! Native copy-constructor unwind leaves completed nodes alive, unlike ordinary assignment.
use super::{BaseSequence, Sequence};

struct IncompleteSequenceCopy<'a, T> {
    destination: Option<&'a mut Sequence<T>>,
}
impl<T> Drop for IncompleteSequenceCopy<'_, T> {
    fn drop(&mut self) {
        let Some(destination) = self.destination.take() else {
            return;
        };
        // Invalidate topology before relinquishing allocation leases: no ordinary local
        // cleanup or observer may traverse nodes after their allocator can reclaim bytes.
        destination.topology.edit(|base| {
            *base = BaseSequence::new(Some(base.allocator().clone()));
        });
        for (_, owner) in destination.owners.drain() {
            owner.abandon();
        }
    }
}
impl<T> Sequence<T> {
    /// Copies with an explicit constructor, without requiring Clone; `Sequence::from_copied_with(&s, |x| *x)`.
    /// Like native construction, a callback panic leaves completed node payloads allocated
    /// without running their destructors. The failed current allocation is freed. Their Rust
    /// allocator leases/handles are relinquished, so an arena reset can reclaim the raw bytes.
    /// Use assignment into a live receiver when ownership of a failed prefix must be retained.
    pub fn from_copied_with(source: &Self, mut construct: impl FnMut(&T) -> T) -> Self {
        let mut destination = Self::with_allocator(source.allocator().clone());
        let mut incomplete = IncompleteSequenceCopy {
            destination: Some(&mut destination),
        };
        for value in source.iter() {
            incomplete
                .destination
                .as_deref_mut()
                .unwrap()
                .emplace_append(|| construct(value));
        }
        incomplete.destination = None;
        drop(incomplete);
        destination
    }
}

#[cfg(test)]
pub(super) mod tests;
