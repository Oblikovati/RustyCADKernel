//! Actual compound subclass reads authoritative old Value fields from its native destructor callback.
use super::AisColoredDrawerCompounds;
use rustycad_tkbrep::topods::{TCompound, TShape, TShapeBase};
use rustycad_tkernel::standard::{Failure, Handle, HandleObserver};
use std::{
    fmt::Write,
    sync::{Arc, Mutex, Weak},
};
pub(super) struct ColoredCompoundValueDropFake {
    parent: TCompound,
    owner: Weak<AisColoredDrawerCompounds>,
    incoming: HandleObserver<dyn TShape>,
    rows: Arc<Mutex<String>>,
}
rustycad_tkernel::impl_transient!(ColoredCompoundValueDropFake,"CompoundValueDestructorProbe",TCompound,parent,interfaces:[dyn TShape]);
impl ColoredCompoundValueDropFake {
    pub(super) fn new(
        owner: &Arc<AisColoredDrawerCompounds>,
        incoming: &Handle<dyn TShape>,
        rows: &Arc<Mutex<String>>,
    ) -> Handle<Self> {
        Handle::new(Self {
            parent: TCompound::new(),
            owner: Arc::downgrade(owner),
            incoming: incoming.observe(),
            rows: Arc::clone(rows),
        })
    }
}
impl TShape for ColoredCompoundValueDropFake {
    fn tshape_base(&self) -> &TShapeBase {
        let parent = &self.parent;
        parent.tshape_base()
    }
    fn empty_copy(&self) -> Result<Handle<dyn TShape>, Failure> {
        let parent = &self.parent;
        parent.empty_copy()
    }
}
impl Drop for ColoredCompoundValueDropFake {
    fn drop(&mut self) {
        let owner = self
            .owner
            .upgrade()
            .expect("map expired; expected live assignment owner");
        let node = owner.node_ref(1).unwrap().live_node().unwrap();
        let old_address = node.compound.object_address();
        let same = old_address == std::ptr::from_ref(self).cast();
        writeln!(
            self.rows.lock().unwrap(),
            "value-destructor {} {} {} {}",
            owner.len(),
            i32::from(same),
            node.compound.ref_count(),
            self.incoming.ref_count()
        )
        .unwrap();
    }
}
