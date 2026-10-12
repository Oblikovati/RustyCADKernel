//! Validate actual node ownership before following native metadata during controlled destruction.
use super::{BaseSequence, SeqNode};
use crate::standard::{exceptions::program_error, raise_native_failure};

#[derive(Clone, Copy)]
pub(in crate::ncollection::sequence) struct SequenceNodeLiveness {
    pub(in crate::ncollection::sequence) registry: *const (),
    pub(in crate::ncollection::sequence) contains: unsafe fn(*const (), usize) -> bool,
}
// SAFETY: Sequence borrows/adapter guards exclude owner-map mutation during each topology call.
unsafe impl Send for SequenceNodeLiveness {}
unsafe impl Sync for SequenceNodeLiveness {}
impl BaseSequence {
    pub(super) fn require_live_link(&self, link: *mut SeqNode) {
        let Some(scope) = self.clear_node_scope else {
            return;
        };
        // SAFETY: the controlled-clear scope retains its stable registry for this entire call.
        if unsafe { (scope.contains)(scope.registry, link as usize) } {
            return;
        }
        raise_native_failure::<()>(Err(program_error(&format!(
            "sequence node {link:p}, size {}; expected a live registered node",
            self.size
        ))));
    }
    pub(super) unsafe fn guarded_link(&self, link: *mut SeqNode) -> &SeqNode {
        self.require_live_link(link);
        // SAFETY: an installed scope checked ownership; raw callers retain documented live links.
        unsafe { &*link }
    }
    pub(super) unsafe fn guarded_link_mut(&mut self, link: *mut SeqNode) -> &mut SeqNode {
        self.require_live_link(link);
        // SAFETY: callers own exclusive topology access and validated/documented live allocations.
        unsafe { &mut *link }
    }
}
