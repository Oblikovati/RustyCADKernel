//! Native removal is split at the user-destructor boundary without duplicating metadata.
use super::{BaseListRawIterator, BaseListStorage, ListLinkNode};
#[derive(Clone, Copy)]
pub(in crate::ncollection) struct ListRemovalPhase {
    pub(in crate::ncollection) node: *mut ListLinkNode,
    pub(in crate::ncollection) next: *mut ListLinkNode,
    previous: *mut ListLinkNode,
}
impl BaseListStorage {
    pub(in crate::ncollection) unsafe fn detach_removal(
        &mut self,
        cursor: BaseListRawIterator,
    ) -> ListRemovalPhase {
        // SAFETY: caller supplies live current and its actual predecessor in this exclusive chain.
        let next = unsafe { (*cursor.current).next() };
        if cursor.previous.is_null() {
            self.first = next;
        } else {
            unsafe { *(*cursor.previous).next_mut() = next };
        }
        ListRemovalPhase {
            node: cursor.current,
            next,
            previous: cursor.previous,
        }
    }
    pub(in crate::ncollection) fn finish_removal(&mut self, phase: ListRemovalPhase) {
        if phase.previous.is_null() {
            self.length = self.length.wrapping_sub(1);
            if self.length == 0 {
                self.last = std::ptr::null_mut();
            }
            return;
        }
        if phase.next.is_null() {
            self.last = phase.previous;
        }
        self.length = self.length.wrapping_sub(1);
    }
}
