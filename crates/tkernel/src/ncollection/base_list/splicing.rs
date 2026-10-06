//! Native transfer operations relink externally owned nodes without copying them.
use super::{BaseListRawIterator, BaseListStorage, Failure, ListLinkNode, require_position};
impl BaseListStorage {
    /// Appends another chain and empties its base fields; `unsafe { destination.append_list(&mut source) }`.
    ///
    /// # Safety
    /// Both chains must be valid, disjoint and exclusively linked; the caller retains responsibility for allocator-compatible deletion.
    pub unsafe fn append_list(&mut self, source: &mut Self) {
        if source.is_empty() {
            return;
        }
        if self.is_empty() {
            self.first = source.first;
        } else {
            // SAFETY: the destination tail is live and exclusive under the caller's contract.
            unsafe {
                *(*self.last).next_mut() = source.first;
            }
        }
        self.last = source.last;
        self.length = self.length.wrapping_add(source.length);
        source.reset_links();
    }
    /// Prepends another chain and empties its base fields; `unsafe { destination.prepend_list(&mut source) }`.
    ///
    /// # Safety
    /// Both chains must be valid, disjoint and exclusively linked; deletion must use compatible allocator resources.
    pub unsafe fn prepend_list(&mut self, source: &mut Self) {
        if source.is_empty() {
            return;
        }
        if self.is_empty() {
            self.last = source.last;
        } else {
            // SAFETY: the source tail is a live, exclusive node.
            unsafe {
                *(*source.last).next_mut() = self.first;
            }
        }
        self.first = source.first;
        self.length = self.length.wrapping_add(source.length);
        source.reset_links();
    }
    /// Inserts before the cursor and updates its predecessor; `unsafe { storage.insert_before_node(node, &mut cursor)? }`.
    ///
    /// # Safety
    /// The chain/cursor must be consistent and exclusive, and node must be live, writable and disjoint from the chain.
    pub unsafe fn insert_before_node(
        &mut self,
        node: *mut ListLinkNode,
        cursor: &mut BaseListRawIterator,
    ) -> Result<(), Failure> {
        require_position(cursor, "pInsertBefore")?;
        if cursor.previous.is_null() {
            // SAFETY: caller's exclusive node and chain meet prepend's contract.
            unsafe {
                self.prepend_node(node);
            }
        } else {
            // SAFETY: caller guarantees the node and predecessor are live and writable.
            unsafe {
                *(*cursor.previous).next_mut() = node;
                *(*node).next_mut() = cursor.current;
            }
            self.length = self.length.wrapping_add(1);
        }
        cursor.previous = node;
        Ok(())
    }
    /// Inserts after the cursor, preserving its two links; `unsafe { storage.insert_after_node(node, &mut cursor)? }`.
    ///
    /// # Safety
    /// The cursor must belong to this valid exclusive chain; node must be live, writable and disjoint.
    pub unsafe fn insert_after_node(
        &mut self,
        node: *mut ListLinkNode,
        cursor: &mut BaseListRawIterator,
    ) -> Result<(), Failure> {
        require_position(cursor, "pInsertAfter")?;
        if cursor.current == self.last {
            // SAFETY: caller's exclusive node and chain meet append's contract.
            unsafe {
                self.append_node(node);
            }
            return Ok(());
        }
        // SAFETY: both current and new nodes are exclusive and live.
        unsafe {
            *(*node).next_mut() = (*cursor.current).next();
            *(*cursor.current).next_mut() = node;
        }
        self.length = self.length.wrapping_add(1);
        Ok(())
    }
    /// Transfers a chain before the cursor, retaining the donor allocator; `unsafe { storage.insert_list_before(&mut source, &mut cursor)? }`.
    ///
    /// # Safety
    /// Both chains must be valid, disjoint and exclusive, and cursor must contain its actual destination predecessor.
    pub unsafe fn insert_list_before(
        &mut self,
        source: &mut Self,
        cursor: &mut BaseListRawIterator,
    ) -> Result<(), Failure> {
        require_position(cursor, "pInsertBefore")?;
        if cursor.previous.is_null() {
            cursor.previous = source.last;
            // SAFETY: caller supplies disjoint exclusive valid chains.
            unsafe { self.prepend_list(source) };
            return Ok(());
        }
        if source.is_empty() {
            return Ok(());
        }
        // SAFETY: caller supplies a valid predecessor and nonempty source chain.
        unsafe { self.transfer_before(source, cursor) };
        Ok(())
    }
    /// Transfers a chain after the cursor; `unsafe { storage.insert_list_after(&mut source, &mut cursor)? }`.
    ///
    /// # Safety
    /// Both chains must be valid, disjoint and exclusive; current must be a live destination node unless it is end.
    pub unsafe fn insert_list_after(
        &mut self,
        source: &mut Self,
        cursor: &mut BaseListRawIterator,
    ) -> Result<(), Failure> {
        require_position(cursor, "pInsertAfter")?;
        if cursor.current == self.last {
            // SAFETY: caller supplies disjoint exclusive valid chains.
            unsafe { self.append_list(source) };
            return Ok(());
        }
        if source.is_empty() {
            return Ok(());
        }
        // SAFETY: caller supplies a valid current node and nonempty source chain.
        unsafe { self.transfer_after(source, cursor) };
        Ok(())
    }
    unsafe fn transfer_before(&mut self, source: &mut Self, cursor: &mut BaseListRawIterator) {
        // SAFETY: predecessor and source tail are exclusive live nodes.
        unsafe {
            *(*cursor.previous).next_mut() = source.first;
            *(*source.last).next_mut() = cursor.current;
        }
        cursor.previous = source.last;
        self.length = self.length.wrapping_add(source.length);
        source.reset_links();
    }
    unsafe fn transfer_after(&mut self, source: &mut Self, cursor: &mut BaseListRawIterator) {
        // SAFETY: current and source tail are exclusive live nodes.
        unsafe {
            *(*source.last).next_mut() = (*cursor.current).next();
            *(*cursor.current).next_mut() = source.first;
        }
        self.length = self.length.wrapping_add(source.length);
        source.reset_links();
    }
}
