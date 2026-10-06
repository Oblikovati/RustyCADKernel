use super::{List, ListCursor, find_link, positions::locate};
use crate::standard::Failure;
impl<T: Clone> List<T> {
    /// Copies contents while preserving this allocator (OCC25348); `destination.assign(&source)`.
    pub fn assign(&mut self, source: &Self) -> &mut Self {
        self.clear();
        for value in source {
            self.append_ref(value);
        }
        self
    }
    /// Appends and empties another list; `destination.append_list(&mut source)`.
    pub fn append_list(&mut self, source: &mut Self) {
        if self.resource != source.resource {
            self.append_copied_list(source);
            return;
        }
        let tail = find_link(&mut self.head, None);
        *tail = source.head.take();
        self.count += std::mem::take(&mut source.count);
    }
    /// Prepends a list, panicking on native failure; `destination.prepend_list(&mut source)`.
    /// Use `prepend_list_checked` to handle a nonempty foreign source and empty destination.
    pub fn prepend_list(&mut self, source: &mut Self) {
        self.prepend_list_checked(source).expect(
            "native list prepend failed; expected a nonempty destination for a foreign allocator",
        );
    }
    /// Preserves native cross-allocator prepend failures; `destination.prepend_list_checked(&mut source)?`.
    pub fn prepend_list_checked(&mut self, source: &mut Self) -> Result<(), Failure> {
        if self.resource != source.resource {
            return self.insert_copied_list_before(source, &self.cursor());
        }
        let tail = find_link(&mut source.head, None);
        *tail = self.head.take();
        self.head = source.head.take();
        self.count += std::mem::take(&mut source.count);
        Ok(())
    }
    /// Splices a list before a positioned node, emptying the source; an end cursor reports
    /// NoSuchObject except for an empty source in another allocator, which OCCT copies
    /// without ever checking the position; `destination.insert_list_before(&mut source, &cursor)?`.
    pub fn insert_list_before(
        &mut self,
        source: &mut Self,
        cursor: &ListCursor,
    ) -> Result<(), Failure> {
        if self.resource != source.resource {
            return self.insert_copied_list_before(source, cursor);
        }
        locate(&mut self.head, "InsertBefore", *cursor)?;
        let link = find_link(&mut self.head, cursor.address);
        let tail = find_link(&mut source.head, None);
        *tail = link.take();
        *link = source.head.take();
        self.count += std::mem::take(&mut source.count);
        Ok(())
    }
    /// Splices a list after a positioned node, emptying the source; an end cursor appends
    /// (OCCT's `InsertAfter(List&, Iterator&)`); `destination.insert_list_after(&mut source, &cursor)?`.
    pub fn insert_list_after(
        &mut self,
        source: &mut Self,
        cursor: &ListCursor,
    ) -> Result<(), Failure> {
        if !cursor.more() {
            self.append_list(source);
            return Ok(());
        }
        locate(&mut self.head, "InsertAfter", *cursor)?;
        if self.resource != source.resource {
            let mut following = *cursor;
            following.next(self);
            return self.insert_copied_list_before(source, &following);
        }
        self.splice_list_after(source, cursor);
        Ok(())
    }
    fn splice_list_after(&mut self, source: &mut Self, cursor: &ListCursor) {
        let node = find_link(&mut self.head, cursor.address)
            .as_mut()
            .expect("located links hold a node");
        let rest = node.next.take();
        node.next = source.head.take();
        *find_link(&mut node.next, None) = rest;
        self.count += std::mem::take(&mut source.count);
    }
    fn append_copied_list(&mut self, source: &mut Self) {
        // Native appendList commits each node before constructing the next one.
        for value in source.iter() {
            self.append_ref(value);
        }
        source.clear();
    }
    fn insert_copied_list_before(
        &mut self,
        source: &mut Self,
        cursor: &ListCursor,
    ) -> Result<(), Failure> {
        // Native checks the cursor after each copy; an empty source never checks it.
        for value in source.iter() {
            self.insert_before_ref(value, cursor)?;
        }
        source.clear();
        Ok(())
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        let mut copy = Self::with_allocator(self.resource.clone());
        copy.assign(self);
        copy
    }
    fn clone_from(&mut self, source: &Self) {
        // OCC25348: replacement retains the destination allocator, unlike construction.
        self.assign(source);
    }
}
