//! The native exhausted index is captured capacity, including arrays with no live slots.
use super::SparseArray;
use crate::{ncollection::base_map::exhausted_iterator, standard::Failure};

pub(super) fn exhausted_sparse_cursor(method: &str, key: usize) -> Failure {
    exhausted_iterator(&format!("{method} at key {key}"))
}

#[derive(Clone, Copy, Default)]
pub(super) struct SparseCursorPosition {
    pub(super) index: usize,
    capacity: usize,
}

impl SparseCursorPosition {
    pub(super) fn new<T>(array: &SparseArray<T>, start: usize) -> Self {
        let capacity = array.capacity();
        Self {
            // Native skipUnset keeps an explicitly requested index at or beyond myEnd.
            index: if start >= capacity {
                start
            } else {
                first_sparse_key(array, start).unwrap_or(capacity)
            },
            capacity,
        }
    }

    pub(super) fn more(&self) -> bool {
        self.index < self.capacity
    }

    pub(super) fn advance<T>(&mut self, array: Option<&SparseArray<T>>) {
        if !self.more() {
            return;
        }
        self.index = array
            .and_then(|array| first_sparse_key(array, self.index + 1))
            .unwrap_or(self.capacity);
    }

    pub(super) fn restart<T>(&mut self, array: Option<&SparseArray<T>>) {
        self.index = array
            .and_then(|array| first_sparse_key(array, 0))
            .unwrap_or(self.capacity);
    }
}

fn first_sparse_key<T>(array: &SparseArray<T>, start: usize) -> Option<usize> {
    if start >= array.capacity() {
        return None;
    }
    array.storage.first_key(start)
}
