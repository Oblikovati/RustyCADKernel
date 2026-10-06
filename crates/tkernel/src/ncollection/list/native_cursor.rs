//! A native cursor's canonical state can be observed during removal without aliasing a mutable cursor.
use super::{BaseListRawIterator, List, ListCursor, address, removal::resume_drop_outcome};
use crate::standard::Failure;
use std::sync::{Arc, Mutex, PoisonError};
struct NativeCursorState(Mutex<BaseListRawIterator>);
// SAFETY: cursor links are opaque identities; dereferencing always requires an owning List borrow.
unsafe impl Send for NativeCursorState {}
unsafe impl Sync for NativeCursorState {}
impl NativeCursorState {
    fn read(&self) -> BaseListRawIterator {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
    fn write(&self, raw: BaseListRawIterator) {
        *self.0.lock().unwrap_or_else(PoisonError::into_inner) = raw;
    }
}
/// Owned native cursor state; unlike the Copy `ListCursor`, it supports live observer handles.
/// Example: `let mut cursor = ListNativeCursor::new(&list); list.remove_native(&mut cursor)?`.
/// Cloning copies native links into independent state; it does not share cursor mutation.
pub struct ListNativeCursor {
    shared: Arc<NativeCursorState>,
}
/// Read-only access to a native cursor's actual current/previous links.
/// Example: `let observer = cursor.observer(); assert!(observer.more())`.
#[derive(Clone)]
pub struct ListCursorObserver {
    shared: Arc<NativeCursorState>,
}
impl ListNativeCursor {
    /// Starts at the owner's head, with null previous; `ListNativeCursor::new(&list)`.
    pub fn new<T>(list: &List<T>) -> Self {
        Self::from_raw(BaseListRawIterator::new(list.base_storage()))
    }
    fn from_raw(raw: BaseListRawIterator) -> Self {
        Self {
            shared: Arc::new(NativeCursorState(Mutex::new(raw))),
        }
    }
    /// Creates a live read-only view without borrowing this cursor; `cursor.observer()`.
    pub fn observer(&self) -> ListCursorObserver {
        ListCursorObserver {
            shared: self.shared.clone(),
        }
    }
    /// Returns the current detached safe cursor expression; `cursor.position().value(&list)`.
    pub fn position(&self) -> ListCursor {
        let raw = self.shared.read();
        ListCursor {
            address: address(raw.current),
            previous: address(raw.previous),
        }
    }
    /// Advances native links after checking live owner membership; `cursor.next(&list)?`.
    pub fn next<T>(&mut self, list: &List<T>) -> Result<(), Failure> {
        let mut raw = self.shared.read();
        if raw.current.is_null() {
            return Ok(());
        }
        let actual = list.locate_cursor(self.position(), "Next")?;
        raw.previous = actual.current;
        raw.current = unsafe { (*actual.current).next() };
        self.shared.write(raw);
        Ok(())
    }
}
impl Default for ListNativeCursor {
    fn default() -> Self {
        Self::from_raw(BaseListRawIterator::default())
    }
}
impl Clone for ListNativeCursor {
    fn clone(&self) -> Self {
        Self::from_raw(self.shared.read())
    }
}
impl ListCursorObserver {
    /// Reads native More without dereferencing current; `observer.more()`.
    pub fn more(&self) -> bool {
        self.shared.read().more()
    }
    /// Reads opaque current identity; `observer.current_node()`.
    pub fn current_node(&self) -> Option<usize> {
        address(self.shared.read().current)
    }
    /// Reads remembered predecessor identity, including at end; `observer.previous_node()`.
    pub fn previous_node(&self) -> Option<usize> {
        address(self.shared.read().previous)
    }
}
impl<T> List<T> {
    /// Removes through an observable canonical cursor; `list.remove_native(&mut cursor)?`.
    pub fn remove_native(&mut self, cursor: &mut ListNativeCursor) -> Result<(), Failure> {
        let raw = cursor.shared.read();
        let actual = self.locate_cursor(cursor.position(), "Remove")?;
        if raw.previous != actual.previous {
            return Err(super::no_such_object(
                "Remove",
                "a cursor with stale previous link",
            ));
        }
        let (next, outcome) = self.remove_in_phases(actual);
        cursor.shared.write(BaseListRawIterator {
            current: next,
            previous: actual.previous,
        });
        resume_drop_outcome(outcome);
        Ok(())
    }
}
