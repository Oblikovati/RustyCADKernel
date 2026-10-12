//! Allocated indexed-map nodes keep native Key/Value slots stable across table growth.
use super::{AisColoredDrawer, colored_compound_value::ColoredCompoundValue};
use rustycad_tkbrep::topods::Compound;
use rustycad_tkernel::{
    ncollection::HashPolicy,
    standard::{Failure, Handle, IntrusiveReferenceSlot},
};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub(super) struct ColoredCompoundNode {
    pub drawer: IntrusiveReferenceSlot<AisColoredDrawer>,
    pub compound: ColoredCompoundValue,
    live: AtomicBool,
}
impl ColoredCompoundNode {
    pub(super) fn new(drawer: &Handle<AisColoredDrawer>, compound: &Compound) -> Self {
        // OCCT 3d097a0 initializes TListNode<Value> before retaining Key.
        let compound = ColoredCompoundValue::new(compound);
        let key = IntrusiveReferenceSlot::default();
        key.assign(drawer)
            .expect("drawer key assignment failed; expected an idle new slot");
        Self {
            drawer: key,
            compound,
            live: AtomicBool::new(true),
        }
    }
    pub(super) fn check_live(&self) -> Result<(), Failure> {
        if !self.live.load(Ordering::Relaxed) {
            return Err(colored_compound_removed());
        }
        Ok(())
    }
    pub(super) fn release(&self) {
        // Value remains live during Key destructor callbacks; invalidate metadata only after both releases.
        self.drawer
            .clear()
            .expect("drawer key retirement failed; expected one owning-map release");
        self.compound.release();
        self.live.store(false, Ordering::Relaxed);
    }
}
impl fmt::Debug for ColoredCompoundNode {
    fn fmt(&self, stream: &mut fmt::Formatter<'_>) -> fmt::Result {
        let live = self.live.load(Ordering::Relaxed);
        stream
            .debug_struct("ColoredCompoundNode")
            .field("live", &live)
            .finish()
    }
}
#[derive(Clone, Copy, Default)]
pub(super) struct ColoredCompoundHasher;
impl HashPolicy<Arc<ColoredCompoundNode>> for ColoredCompoundHasher {
    fn hash(&self, node: &Arc<ColoredCompoundNode>) -> usize {
        // Native std::hash<Handle<T>> hashes the pointer, including the null key.
        node.drawer.object_address() as usize
    }
    fn equal(&self, first: &Arc<ColoredCompoundNode>, second: &Arc<ColoredCompoundNode>) -> bool {
        let first_address = self.hash(first);
        first_address == self.hash(second)
    }
}
pub(super) fn colored_compound_removed() -> Failure {
    Failure::with_type(
        "Standard_NoSuchObject",
        "colored compound node=None; expected a live indexed-map node",
        "",
    )
}
