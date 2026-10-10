//! Detached native list cursors keep callbacks outside selection read guards.
use super::{AisSelectedOwner, AisSelectionSource};
use rustycad_tkernel::{
    ncollection::{List, ListCursor},
    standard::Handle,
};
pub(super) enum AisOwnerList<'a> {
    Borrowed(&'a List<AisSelectedOwner>),
    Selected(Handle<dyn AisSelectionSource>),
}
pub(super) struct AisOwnerWalk<'a> {
    source: AisOwnerList<'a>,
    cursor: ListCursor,
}
impl<'a> AisOwnerWalk<'a> {
    pub(super) fn new(source: AisOwnerList<'a>) -> Self {
        let cursor = match &source {
            AisOwnerList::Borrowed(list) => list.cursor(),
            AisOwnerList::Selected(selection) => {
                selection.selection_base().fields.read().owners.cursor()
            }
        };
        Self { source, cursor }
    }
    pub(super) fn current(&self) -> Option<AisSelectedOwner> {
        match &self.source {
            AisOwnerList::Borrowed(list) => self.cursor.value(list).cloned(),
            AisOwnerList::Selected(selection) => self
                .cursor
                .value(&selection.selection_base().fields.read().owners)
                .cloned(),
        }
    }
    pub(super) fn next(&mut self) {
        match &self.source {
            AisOwnerList::Borrowed(list) => self.cursor.next(list),
            AisOwnerList::Selected(selection) => self
                .cursor
                .next(&selection.selection_base().fields.read().owners),
        }
    }
}
