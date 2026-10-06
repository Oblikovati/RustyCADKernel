//! Standard iterator return contracts shared by const values and legacy pair cursors.
use super::{Failure, FlatDataItems, FlatDataValues};

macro_rules! readonly_flat_cursor_contracts {
    ($cursor:ident) => {
        impl<K, V> $cursor<'_, K, V> {
            /// Returns the current value pointer, as native operator->; `cursor.value_ptr()?`.
            pub fn value_ptr(&self) -> Result<*const V, Failure> {
                self.value().map(std::ptr::from_ref)
            }
            /// Advances and returns this iterator; `cursor.advance_returning().value()`.
            pub fn advance_returning(&mut self) -> &mut Self {
                self.advance();
                self
            }
            /// Returns the previous position while advancing; `cursor.advance_post().value()`.
            pub fn advance_post(&mut self) -> Self {
                let previous = self.clone();
                self.advance();
                previous
            }
            /// Copies slot state and returns this iterator; `cursor.assign_from(&source)`.
            pub fn assign_from(&mut self, source: &Self) -> &mut Self {
                self.inner = source.inner.clone();
                self
            }
            /// Preserves self-assignment without aliased references; `cursor.assign_self()`.
            pub fn assign_self(&mut self) -> &mut Self {
                self
            }
            /// Compares slot identity with all exhausted iterators equal; `cursor.is_equal(&other)`.
            pub fn is_equal(&self, other: &Self) -> bool {
                self == other
            }
        }
    };
}
readonly_flat_cursor_contracts!(FlatDataItems);
readonly_flat_cursor_contracts!(FlatDataValues);

impl<'a, K, V> FlatDataValues<'a, K, V> {
    /// Checks for a current native iterator position; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.inner.more()
    }
    /// Advances the native iterator without yielding its previous value; `cursor.advance()`.
    pub fn advance(&mut self) {
        self.inner.advance();
    }
    /// Borrows the current key through the native iterator Key method; `cursor.key()?`.
    pub fn key(&self) -> Result<&'a K, Failure> {
        FlatDataItems {
            inner: self.inner.clone(),
        }
        .key()
    }
    /// Borrows the current item without advancing, as native operator*; `cursor.value()?`.
    pub fn value(&self) -> Result<&'a V, Failure> {
        FlatDataItems {
            inner: self.inner.clone(),
        }
        .value()
    }
}
