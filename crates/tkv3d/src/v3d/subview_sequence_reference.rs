//! Native subview traversal retains actual Sequence nodes through valid callback edits.
use super::{V3dView, V3dViewSource};
use rustycad_tkernel::{
    ncollection::SequenceCursor,
    standard::{HandleObserver, HandleReference, raise_native_failure},
};
/// Live native subview iterator; `let mut cursor=parent.subview_iterator(); cursor.next()`.
pub struct V3dSubviewCursor<'a> {
    parent: &'a V3dView,
    position: SequenceCursor,
}
/// Actual current subview handle slot; `cursor.value_ref().observe()`.
pub struct V3dSubviewNodeRef<'a> {
    parent: &'a V3dView,
    position: SequenceCursor,
}
impl<'a> V3dSubviewCursor<'a> {
    pub(super) fn new(parent: &'a V3dView) -> Self {
        Self {
            parent,
            position: parent.state.read().subviews.cursor(),
        }
    }
    /// Inspect the current native node; `while cursor.more(){cursor.next();}`.
    pub fn more(&self) -> bool {
        self.position.more()
    }
    /// Borrow the actual current node slot; `let child=cursor.value_ref()`.
    pub fn value_ref(&self) -> V3dSubviewNodeRef<'a> {
        V3dSubviewNodeRef {
            parent: self.parent,
            position: self.position,
        }
    }
    /// Follow live links after callbacks; `cursor.next()`.
    pub fn next(&mut self) {
        let state = self.parent.state.read();
        self.position.next(&state.subviews);
    }
}
impl<'a> V3dSubviewNodeRef<'a> {
    pub(super) fn new(parent: &'a V3dView, position: SequenceCursor) -> Self {
        Self { parent, position }
    }
}
impl HandleReference<dyn V3dViewSource> for V3dSubviewNodeRef<'_> {
    fn observe(&self) -> HandleObserver<dyn V3dViewSource> {
        let state = self.parent.state.read();
        self.position
            .value(&state.subviews)
            .unwrap_or_else(|| invalid_subview_node(self.position))
            .observe()
    }
    fn reference_address(&self) -> *const () {
        let state = self.parent.state.read();
        std::ptr::from_ref(
            self.position
                .value(&state.subviews)
                .unwrap_or_else(|| invalid_subview_node(self.position)),
        )
        .cast()
    }
}

fn invalid_subview_node(position: SequenceCursor) -> ! {
    match raise_native_failure::<std::convert::Infallible>(Err(
        rustycad_tkernel::standard::exceptions::program_error(&format!(
            "V3d subview node={:?}; expected original live sequence node",
            position.current_node()
        )),
    )) {}
}
