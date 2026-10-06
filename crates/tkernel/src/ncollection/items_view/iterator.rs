//! Read-only forward cursors retain a cloneable native-style position without copying items.

/// Cursor over map item references; obtain one with `ItemsView::new(&map).begin()`.
pub struct ItemsIterator<I> {
    inner: Option<I>,
    owner: usize,
    position: usize,
}

impl<I> Default for ItemsIterator<I> {
    fn default() -> Self {
        Self {
            inner: None,
            owner: 0,
            position: 0,
        }
    }
}

impl<I: Clone> Clone for ItemsIterator<I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            owner: self.owner,
            position: self.position,
        }
    }
}

impl<I> ItemsIterator<I> {
    pub(super) fn new(inner: I, owner: usize) -> Self {
        Self {
            inner: Some(inner),
            owner,
            position: 0,
        }
    }
}

impl<I: Iterator + Clone> ItemsIterator<I> {
    /// Extracts the current references without advancing, like `operator*`; `cursor.value()`.
    pub fn value(&self) -> Option<I::Item> {
        self.inner.as_ref()?.clone().next()
    }

    /// Distinguishes a live position from the default end; `cursor.more()`.
    pub fn more(&self) -> bool {
        self.value().is_some()
    }

    /// Advances and returns this cursor, like prefix `++`; `cursor.advance_returning()`.
    pub fn advance_returning(&mut self) -> &mut Self {
        self.next();
        self
    }

    /// Returns the previous cursor then advances, like postfix `++`; `cursor.advance_post()`.
    pub fn advance_post(&mut self) -> Self {
        let previous = self.clone();
        self.advance_returning();
        previous
    }

    /// Copies the forward position and returns the destination; `cursor.assign_from(&other)`.
    pub fn assign_from(&mut self, other: &Self) -> &mut Self {
        self.clone_from(other);
        self
    }

    /// Preserves the current position for native self-assignment; `cursor.assign_self()`.
    pub fn assign_self(&mut self) -> &mut Self {
        self
    }
}

impl<I: Iterator> Iterator for ItemsIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.as_mut()?.next()?;
        self.position += 1;
        Some(item)
    }
}

impl<I: Iterator + Clone, J: Iterator + Clone> PartialEq<ItemsIterator<J>> for ItemsIterator<I> {
    fn eq(&self, other: &ItemsIterator<J>) -> bool {
        match (self.more(), other.more()) {
            (false, false) => true,
            (true, true) => self.owner == other.owner && self.position == other.position,
            _ => false,
        }
    }
}

impl<I: Iterator + Clone> Eq for ItemsIterator<I> {}
