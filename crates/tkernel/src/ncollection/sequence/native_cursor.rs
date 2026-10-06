//! Canonical native cursor state can advance before Drop without aliasing its observer.
use super::{
    BaseSequenceRawIterator, Sequence, SequenceCursor, iteration::pointer_address,
    owner::take_owner,
};
use crate::standard::Failure;
use std::sync::{Arc, Mutex, PoisonError};
struct SequenceCursorState(Mutex<BaseSequenceRawIterator>);
// SAFETY: cursor pointers are identities only; dereferencing requires fresh owner lookup.
unsafe impl Send for SequenceCursorState {}
unsafe impl Sync for SequenceCursorState {}
impl SequenceCursorState {
    fn read(&self) -> BaseSequenceRawIterator {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn write(&self, cursor: BaseSequenceRawIterator) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = cursor;
    }
}
/// Observable native cursor; `let mut cursor = SequenceNativeCursor::new(&sequence, true)`.
/// Clone copies links independently; observer() shares this cursor's actual state.
pub struct SequenceNativeCursor {
    shared: Arc<SequenceCursorState>,
}
/// Read-only canonical cursor links; `let observer = cursor.observer(); observer.more()`.
#[derive(Clone)]
pub struct SequenceCursorObserver {
    shared: Arc<SequenceCursorState>,
}
impl SequenceNativeCursor {
    /// Positions at first or past-last; `SequenceNativeCursor::new(&sequence, false)`.
    pub fn new<T>(sequence: &Sequence<T>, start: bool) -> Self {
        Self::from_raw(BaseSequenceRawIterator::new(&sequence.base_access(), start))
    }
    fn from_raw(raw: BaseSequenceRawIterator) -> Self {
        Self {
            shared: Arc::new(SequenceCursorState(Mutex::new(raw))),
        }
    }
    /// Creates a view of actual current/previous fields; `cursor.observer()`.
    pub fn observer(&self) -> SequenceCursorObserver {
        SequenceCursorObserver {
            shared: self.shared.clone(),
        }
    }
    /// Copies the current safe detached position; `cursor.position().value(&sequence)`.
    pub fn position(&self) -> SequenceCursor {
        let raw = self.shared.read();
        SequenceCursor {
            address: pointer_address(raw.current),
            previous: pointer_address(raw.previous),
        }
    }
    /// Advances from a freshly resolved live node; `cursor.next(&sequence)?`.
    pub fn next<T>(&mut self, sequence: &Sequence<T>) -> Result<(), Failure> {
        let position = self.position();
        if !position.more() {
            return Ok(());
        }
        let node = sequence
            .owned_node(position.address)
            .ok_or_else(|| super::no_such_node("Next", position))?;
        self.shared.write(BaseSequenceRawIterator {
            current: unsafe { (*node).next() },
            previous: node,
        });
        Ok(())
    }
    /// Steps through the remembered live predecessor; `cursor.previous(&sequence)?`.
    pub fn previous<T>(&mut self, sequence: &Sequence<T>) -> Result<(), Failure> {
        let position = self.position();
        let Some(address) = position.previous else {
            self.shared.write(BaseSequenceRawIterator::default());
            return Ok(());
        };
        let node = sequence
            .owned_node(Some(address))
            .ok_or_else(|| super::no_such_node("Previous", position))?;
        self.shared.write(BaseSequenceRawIterator {
            current: node,
            previous: unsafe { (*node).previous() },
        });
        Ok(())
    }
}
impl Default for SequenceNativeCursor {
    fn default() -> Self {
        Self::from_raw(BaseSequenceRawIterator::default())
    }
}
impl Clone for SequenceNativeCursor {
    fn clone(&self) -> Self {
        Self::from_raw(self.shared.read())
    }
}
impl SequenceCursorObserver {
    /// Reads native More without dereferencing current; `observer.more()`.
    pub fn more(&self) -> bool {
        !self.shared.read().current.is_null()
    }
    /// Reads opaque current identity; `observer.current_node()`.
    pub fn current_node(&self) -> Option<usize> {
        pointer_address(self.shared.read().current)
    }
    /// Reads remembered opaque predecessor identity; `observer.previous_node()`.
    pub fn previous_node(&self) -> Option<usize> {
        pointer_address(self.shared.read().previous)
    }
}
impl<T> Sequence<T> {
    /// Updates native topology/cache/cursor before item Drop; `sequence.remove_native(&mut cursor)?`.
    pub fn remove_native(&mut self, cursor: &mut SequenceNativeCursor) -> Result<(), Failure> {
        let position = cursor.position();
        if !position.more() {
            return Ok(());
        }
        let node = self
            .owned_node(position.address)
            .ok_or_else(|| super::no_such_node("Remove", position))?;
        let mut raw = cursor.shared.read();
        raw.current = node;
        self.prepare_owned_cursor(&mut raw);
        cursor.shared.write(raw);
        drop(take_owner(&mut self.owners, node));
        Ok(())
    }
}
