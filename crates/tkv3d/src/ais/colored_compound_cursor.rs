//! Native indexed-map iteration rereads Size on every More instead of caching the original extent.
use super::{AisColoredCompoundRef, AisColoredDrawerCompounds};
use rustycad_tkernel::standard::Failure;
pub(super) struct ColoredCompoundNativeCursor<'a> {
    owner: &'a AisColoredDrawerCompounds,
    index: usize,
}
impl<'a> ColoredCompoundNativeCursor<'a> {
    pub(super) fn new(owner: &'a AisColoredDrawerCompounds) -> Self {
        let index = 1;
        Self { owner, index }
    }
    pub(super) fn current(&self) -> Result<Option<AisColoredCompoundRef<'a>>, Failure> {
        if self.index > self.owner.len() {
            return Ok(None);
        }
        self.owner.node_ref(self.index).map(Some)
    }
    pub(super) fn next(&mut self) {
        // Native permits an initially empty cursor to see later Add; Next only advances its integer index.
        self.index += 1;
    }
}
