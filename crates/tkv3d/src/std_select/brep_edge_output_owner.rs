//! Named native-equivalent owner destructor observes pointer retirement without projecting a dying entity.
use crate::{select_mgr::*, select3d::Select3DSensitiveEntity};
use rustycad_tkernel::standard::{
    Handle, IntrusiveReferenceSlot, SharedState, StandardTransient, WeakHandle,
};
use std::{
    fmt::Write,
    sync::{Arc, Weak},
};
pub(super) struct EdgeOutputRetirementInput {
    pub slot: Weak<IntrusiveReferenceSlot<dyn Select3DSensitiveEntity>>,
    pub incoming: WeakHandle<dyn SelectMgrEntityOwnerSource>,
    pub previous: SharedState<WeakHandle<dyn Select3DSensitiveEntity>>,
    pub rows: Arc<SharedState<String>>,
}
pub(super) struct EdgeOutputRetiringOwner {
    transient: StandardTransient,
    owner: SelectMgrEntityOwnerBase,
    input: Arc<EdgeOutputRetirementInput>,
}
rustycad_tkernel::impl_transient!(EdgeOutputRetiringOwner,"Martensite_EdgeOutputRetiringOwner",dyn SelectMgrEntityOwnerSource,transient,interfaces:[dyn SelectMgrEntityOwnerSource]);
impl EdgeOutputRetiringOwner {
    pub fn new(input: &Arc<EdgeOutputRetirementInput>) -> Handle<Self> {
        Handle::new(Self {
            transient: Default::default(),
            owner: SelectMgrEntityOwnerBase::new(&Handle::default(), 5),
            input: input.clone(),
        })
    }
}
impl SelectMgrEntityOwnerSource for EdgeOutputRetiringOwner {
    fn owner_base(&self) -> &SelectMgrEntityOwnerBase {
        // This is the actual inherited owner state destroyed after the derived callback.
        &self.owner
    }
}
impl Drop for EdgeOutputRetiringOwner {
    fn drop(&mut self) {
        let slot = self.input.slot.upgrade().unwrap();
        let previous = self.input.previous.read();
        let same = i32::from(slot.object_address() == previous.object_address());
        // No projection of the dying sensitive entity is attempted: its ownership count has reached zero.
        let count = previous.observe().ref_count();
        let incoming = self.input.incoming.observe().ref_count();
        writeln!(
            &mut *self.input.rows.write(),
            "output-retiring {same} {count} {incoming}"
        )
        .unwrap();
    }
}
