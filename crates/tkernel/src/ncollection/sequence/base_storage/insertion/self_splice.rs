//! Native raw self-splice drops metadata after mutating links; typed Sequence forbids this path.
use super::BaseSequence;
use crate::standard::Failure;

impl BaseSequence {
    /// Reproduces raw native PAppend(self), retaining the resulting cycle in caller-owned nodes.
    /// Example: `unsafe { base.append_self() }; assert!(base.is_empty())`.
    /// # Safety
    /// Nodes must be live, exclusively managed and well formed. Caller must retain their
    /// allocation owners and restore/delete them separately after this operation loses all links.
    pub unsafe fn append_self(&mut self) {
        if self.size == 0 {
            return;
        }
        unsafe {
            (*self.last).set_next(self.first);
            (*self.first).set_previous(self.last);
        }
        self.nullify();
    }
    /// Reproduces raw native PPrepend(self); `unsafe { base.prepend_self() }`.
    /// # Safety
    /// The same live-node, exclusive-access and retained-ownership requirements as append_self apply.
    pub unsafe fn prepend_self(&mut self) {
        unsafe { self.append_self() };
    }
    /// Reproduces raw native PInsertAfter(index,self), including its interior cycles.
    /// Example: `unsafe { base.insert_self_after(1)? }; assert!(base.is_empty())`.
    /// # Safety
    /// All node/cache links must be live and consistent. Caller retains every allocation and
    /// must restore/delete nodes separately after this operation clears base ownership metadata.
    pub unsafe fn insert_self_after(&mut self, index: usize) -> Result<(), Failure> {
        self.check_insertion(index)?;
        if self.size == 0 {
            return Ok(());
        }
        if index == 0 {
            unsafe { self.prepend_self() };
            return Ok(());
        }
        let before = unsafe { self.find(index) };
        unsafe { self.link_after(before, self.first, self.last) };
        self.nullify();
        Ok(())
    }
}
