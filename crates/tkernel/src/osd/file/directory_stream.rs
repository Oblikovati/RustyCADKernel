//! Owned directory cursors preserve the native demand-driven `readdir` protocol.
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

/// One open directory stream; `cursor.next_name()` reads only the next entry.
pub trait DirectoryEntries: Send {
    /// Returns the next raw name or end-of-stream; `cursor.next_name()`.
    fn next_name(&mut self) -> Option<Vec<u8>>;
}

pub(super) fn buffered_directory_entries(names: Vec<Vec<u8>>) -> Box<dyn DirectoryEntries> {
    Box::new(BufferedDirectoryEntries(names.into()))
}

struct BufferedDirectoryEntries(VecDeque<Vec<u8>>);
impl DirectoryEntries for BufferedDirectoryEntries {
    fn next_name(&mut self) -> Option<Vec<u8>> {
        self.0.pop_front()
    }
}

// Cloning an OCCT iterator copies its stream pointer, not the unread directory entries.
// Share cursor ownership safely so clones never close or access a dangling DIR pointer.
#[derive(Clone)]
pub(super) struct SharedDirectoryEntries(Arc<Mutex<Box<dyn DirectoryEntries>>>);
impl SharedDirectoryEntries {
    pub(super) fn new(cursor: Box<dyn DirectoryEntries>) -> Self {
        Self(Arc::new(Mutex::new(cursor)))
    }
    pub(super) fn next_name(&self) -> Option<Vec<u8>> {
        self.0
            .lock()
            .expect("directory cursor mutex expected unpoisoned")
            .next_name()
    }
}

#[cfg(test)]
mod tests {
    use super::{SharedDirectoryEntries, buffered_directory_entries};

    #[test]
    fn buffered_adapter_preserves_order_and_end_of_stream() {
        let cursor = SharedDirectoryEntries::new(buffered_directory_entries(vec![
            b"a".to_vec(),
            b"b".to_vec(),
        ]));
        assert_eq!(cursor.next_name(), Some(b"a".to_vec()));
        assert_eq!(cursor.clone().next_name(), Some(b"b".to_vec()));
        assert_eq!(cursor.next_name(), None);
        assert_eq!(cursor.next_name(), None);
    }
}
