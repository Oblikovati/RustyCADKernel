use super::{BaseSequenceBasicIterator, BasicSequenceNode};

impl<N: BasicSequenceNode, const M: bool> BaseSequenceBasicIterator<N, M> {
    /// Implements native Next, including an exhausted no-op; `unsafe { cursor.next() }`.
    /// # Safety
    /// Current must be null or a live readable SeqNode through this call.
    pub unsafe fn next(&mut self) {
        unsafe { self.raw.next() };
    }
    /// Implements native Previous, including end-to-tail movement; `unsafe { cursor.previous() }`.
    /// # Safety
    /// Previous must be null or a live readable SeqNode through this call.
    pub unsafe fn previous(&mut self) {
        unsafe { self.raw.previous() };
    }
    /// Implements prefix increment returning this cursor; `unsafe { cursor.advance() }`.
    /// # Safety
    /// Current must satisfy next's live-link contract.
    pub unsafe fn advance(&mut self) -> &mut Self {
        unsafe { self.next() };
        self
    }
    /// Implements postfix increment returning the old cursor; `unsafe { cursor.post_advance() }`.
    /// # Safety
    /// Current must satisfy next's live-link contract.
    pub unsafe fn post_advance(&mut self) -> Self {
        let old = *self;
        unsafe { self.next() };
        old
    }
    /// Implements prefix decrement returning this cursor; `unsafe { cursor.retreat() }`.
    /// # Safety
    /// Previous must satisfy previous's live-link contract.
    pub unsafe fn retreat(&mut self) -> &mut Self {
        unsafe { self.previous() };
        self
    }
    /// Implements postfix decrement returning the old cursor; `unsafe { cursor.post_retreat() }`.
    /// # Safety
    /// Previous must satisfy previous's live-link contract.
    pub unsafe fn post_retreat(&mut self) -> Self {
        let old = *self;
        unsafe { self.previous() };
        old
    }
}
