//! Native destructor timing is retained; caught panics resume after metadata repair.
use super::{
    BaseListStorage, List, ListCursor,
    owner::{catch_list_owner_drop, take_owner},
};
use crate::standard::Failure;
struct ListClearMetadata<'a>(&'a mut BaseListStorage);
impl Drop for ListClearMetadata<'_> {
    fn drop(&mut self) {
        // Safe access must never retain freed pointers after a throwing Rust destructor.
        *self.0 = BaseListStorage::new(Some(self.0.allocator().clone()));
    }
}
impl<T> List<T> {
    /// Removes current and advances to its successor; `list.remove(&mut cursor)?`.
    pub fn remove(&mut self, cursor: &mut ListCursor) -> Result<(), Failure> {
        let mut raw = self.locate_cursor(*cursor, "Remove")?;
        let owners = &mut self.owners;
        let mut outcome = Ok(());
        unsafe {
            self.base.remove(&mut raw, |node, _| {
                outcome = catch_list_owner_drop(owners, node)
            })?
        };
        // Safe membership repair must not rewrite the caller's remembered predecessor.
        cursor.address = super::address(raw.current);
        if let Err(payload) = outcome {
            std::panic::resume_unwind(payload);
        }
        Ok(())
    }
    pub(super) fn clear_owned_chain(&mut self) {
        let mut owners = std::mem::take(&mut self.owners);
        let reset = ListClearMetadata(&mut self.base);
        unsafe { reset.0.clear(|node, _| drop(take_owner(&mut owners, node))) };
        drop(reset);
    }
}
